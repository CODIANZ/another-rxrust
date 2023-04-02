use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use scheduler::IScheduler;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Clone)]
pub struct TimeoutOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  dur: Duration,
  scheduler_ctor: FunctionWrapper<'a, (), Scheduler>,
  err: RxError,
  _item: PhantomData<Item>,
}

impl<'a, Scheduler, Item> TimeoutOp<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  pub fn new<SchedulerCreator>(
    dur: Duration,
    scheduler_ctor: SchedulerCreator,
    err: RxError,
  ) -> TimeoutOp<'a, Scheduler, Item>
  where
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    TimeoutOp {
      dur,
      scheduler_ctor: FunctionWrapper::new(move |_| scheduler_ctor()),
      err,
      _item: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let scheduler_ctor = self.scheduler_ctor.clone();
    let err = self.err.clone();
    let dur = self.dur;
    Observable::create(move |s| {
      let sctl = StreamController::new(s);
      let timer = Arc::new(RwLock::new(None::<Subscription<'a>>));
      let scheduler_ctor = scheduler_ctor.clone();
      let err = err.clone();

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();
      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          {
            let mut timer = timer.write().unwrap();
            if let Some(timer) = &*timer {
              timer.unsubscribe();
            }
            *timer = None;
          }

          sctl_next.sink_next(x);

          {
            let sctl = sctl_next.clone();
            let scheduler_ctor = scheduler_ctor.clone();
            let err = err.clone();
            *timer.write().unwrap() = Some(
              observables::interval(dur, move || scheduler_ctor.call(()))
                .take(1)
                .subscribe(
                  move |_| {
                    sctl.sink_error(err.clone());
                  },
                  junk_error!(),
                  junk_complete!(),
                ),
            );
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn timeout<Scheduler, SchedulerCreator>(
    &self,
    dur: Duration,
    scheduler_ctor: SchedulerCreator,
    err: RxError,
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    TimeoutOp::new(dur, scheduler_ctor, err).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let sbj = subjects::Subject::new();

    sbj
      .observable()
      .timeout(
        time::Duration::from_millis(100),
        schedulers::new_thread_scheduler(),
        generate_error(),
      )
      .subscribe(print_next_fmt!("{}"), print_error!(), print_complete!());

    sbj.next(1);
    thread::sleep(time::Duration::from_millis(10));
    sbj.next(2);
    thread::sleep(time::Duration::from_millis(10));
    sbj.next(3);
    thread::sleep(time::Duration::from_millis(10));
    sbj.next(4);
    thread::sleep(time::Duration::from_millis(200));
    sbj.next(5);

    thread::sleep(time::Duration::from_millis(1000));
  }
}
