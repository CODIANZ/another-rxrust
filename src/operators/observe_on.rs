use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use scheduler::IScheduler;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct ObserveOn<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  scheduler_ctor: FunctionWrapper<'a, (), Scheduler>,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Scheduler, Item> ObserveOn<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  pub fn new<SchedulerCreator>(
    scheduler_ctor: SchedulerCreator,
  ) -> ObserveOn<'a, Scheduler, Item>
  where
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    ObserveOn {
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn observe_on<Scheduler, SchedulerCreator>(
    &self,
    scheduler_ctor: SchedulerCreator,
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    ObserveOn::new(scheduler_ctor).execute(self.clone())
  }
}

#[cfg(all(test, not(feature = "web")))]
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
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
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
    .observe_on(schedulers::new_thread_scheduler());

    o.subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:?}", e),
      || println!("#1 complete"),
    );

    o.subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:?}", e),
      || println!("#2 complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
