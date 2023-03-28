use crate::{
  internals::stream_controller::StreamController,
  prelude::{schedulers::IScheduler, Observable},
};
use std::marker::PhantomData;

pub struct ObserveOnOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  scheduler: Scheduler,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Scheduler, Item> ObserveOnOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
  Item: Clone + Send + Sync,
{
  pub fn new(scheduler: Scheduler) -> ObserveOnOp<'a, Scheduler, Item> {
    ObserveOnOp {
      scheduler,
      _item: PhantomData,
      _lifetime: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let scheduler = self.scheduler.clone();
    Observable::create(move |s| {
      let sctl = StreamController::new(s);
      let source_next = source.clone();

      let scheduler_on_finalize = scheduler.clone();
      sctl.set_on_finalize(move || {
        scheduler_on_finalize.abort();
      });

      scheduler.post(move || {
        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();
        source_next.inner_subscribe(sctl.new_observer(
          move |_, x| {
            sctl_next.sink_next(x);
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |serial| sctl_complete.sink_complete(&serial),
        ));
      });
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

    o.observe_on(schedulers::new_thread_scheduler()).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
