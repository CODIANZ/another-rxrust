use crate::{
  internals::stream_controller::StreamController,
  prelude::{schedulers::IScheduler, Observable},
};
use std::marker::PhantomData;

pub struct SubscribeOnOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  scheduler: Scheduler,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Scheduler, Item> SubscribeOnOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
  Item: Clone + Send + Sync,
{
  pub fn new(scheduler: Scheduler) -> SubscribeOnOp<'a, Scheduler, Item> {
    SubscribeOnOp {
      scheduler,
      _item: PhantomData,
      _lifetime: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let scheduler = self.scheduler.clone();
    Observable::create(move |s| {
      let sctl = StreamController::new(s);

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let source_next = source.clone();

      let scheduler_next = scheduler.clone();
      let scheduler_error = scheduler.clone();
      let scheduler_complete = scheduler.clone();

      let scheduler_on_finalize = scheduler.clone();
      sctl.set_on_finalize(move || {
        scheduler_on_finalize.abort();
      });

      source_next.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          let sctl_next = sctl_next.clone();
          scheduler_next.post(move || {
            sctl_next.sink_next(x.clone());
          });
        },
        move |_, e| {
          let sctl_error = sctl_error.clone();
          scheduler_error.post(move || {
            sctl_error.sink_error(e.clone());
          });
        },
        move |serial| {
          let sctl_complete = sctl_complete.clone();
          scheduler_complete.post(move || {
            sctl_complete.sink_complete(&serial);
          });
        },
      ));
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
        thread::sleep(time::Duration::from_millis(100));
      }
      s.complete();
    });

    o.subscribe_on(schedulers::new_thread_scheduler())
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", e.error),
        || println!("complete"),
      );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
