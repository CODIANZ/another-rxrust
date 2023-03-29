use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use scheduler::IScheduler;
use std::marker::PhantomData;

pub struct SubscribeOnOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  scheduler_ctor: FunctionWrapper<'a, (), Scheduler>,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Scheduler, Item> SubscribeOnOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  pub fn new<SchedulerCreator>(
    scheduler_ctor: SchedulerCreator,
  ) -> SubscribeOnOp<'a, Scheduler, Item>
  where
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    SubscribeOnOp {
      scheduler_ctor: FunctionWrapper::new(move |_| scheduler_ctor()),
      _item: PhantomData,
      _lifetime: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let scheduler_ctor = self.scheduler_ctor.clone();
    Observable::create(move |s| {
      let scheduler = scheduler_ctor.call(());

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
  use crate::tests::common::*;
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
        |e| println!("error {:}", error_to_string(&e)),
        || println!("complete"),
      );
    thread::sleep(time::Duration::from_millis(1000));
  }

  #[test]
  fn multiple() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
        thread::sleep(time::Duration::from_millis(100));
      }
      s.complete();
    })
    .subscribe_on(schedulers::new_thread_scheduler());

    o.subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", error_to_string(&e)),
      || println!("#1 complete"),
    );

    o.subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", error_to_string(&e)),
      || println!("#2 complete"),
    );

    thread::sleep(time::Duration::from_millis(1000));
  }
}
