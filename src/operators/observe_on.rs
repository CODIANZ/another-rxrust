use crate::{
  internals::stream_controller::StreamController,
  prelude::{schedulers::IScheduler, Observable, Subscription},
};
use std::marker::PhantomData;

pub struct ObserveOnOp<Scheduler, Item>
where
  Scheduler: IScheduler + Clone + Send + Sync + 'static,
  Item: Clone + Send + Sync + 'static,
{
  scheduler: Scheduler,
  _item: PhantomData<Item>,
}

impl<Scheduler, Item> ObserveOnOp<Scheduler, Item>
where
  Scheduler: IScheduler + Clone + Send + Sync + 'static,
  Item: Clone + Send + Sync + 'static,
{
  pub fn new(scheduler: Scheduler) -> ObserveOnOp<Scheduler, Item> {
    ObserveOnOp {
      scheduler,
      _item: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<Item>) -> Observable<Item> {
    let _source = source.clone();
    let scheduler = self.scheduler.clone();
    scheduler.start();
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);
      {
        let sctl = sctl.clone();
        let source_next = _source.clone();
        scheduler.post(move || {
          let sctl_next = sctl.clone();
          let sctl_error = sctl.clone();
          let sctl_complete = sctl.clone();
          let serial = sctl.upstream_prepare_serial();
          sctl.upstream_subscribe(
            &serial,
            source_next.subscribe(
              move |x| {
                sctl_next.sink_next(x);
              },
              move |e| {
                sctl_error.sink_error(e);
              },
              move || sctl_complete.sink_complete(&serial),
            ),
          );
        });
      }
      let scheduler = scheduler.clone();
      Subscription::new(move || {
        sctl.finalize();
        scheduler.stop();
      })
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::<i32>::create(|s| {
      for n in 0..5 {
        s.next(n);
        thread::sleep(time::Duration::from_millis(100));
      }
      s.complete();
      Subscription::new(|| {})
    });

    o.observe_on(schedulers::new_thread()).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(500));
  }
}
