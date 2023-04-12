use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use scheduler::IScheduler;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct SubscribeOn<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  scheduler_ctor: FunctionWrapper<'a, (), Scheduler>,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Scheduler, Item> SubscribeOn<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  pub fn new<SchedulerCreator>(
    scheduler_ctor: SchedulerCreator,
  ) -> SubscribeOn<'a, Scheduler, Item>
  where
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    SubscribeOn {
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn subscribe_on<Scheduler, SchedulerCreator>(
    &self,
    scheduler_ctor: SchedulerCreator,
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    SubscribeOn::new(scheduler_ctor).execute(self.clone())
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

    o.subscribe_on(schedulers::new_thread_scheduler())
      .subscribe(
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
    .subscribe_on(schedulers::new_thread_scheduler());

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
