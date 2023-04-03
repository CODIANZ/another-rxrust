use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use scheduler::IScheduler;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Clone)]
pub struct Timeout<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  dur: Duration,
  scheduler_ctor: FunctionWrapper<'a, (), Scheduler>,
  _item: PhantomData<Item>,
}

impl<'a, Scheduler, Item> Timeout<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  pub fn new<SchedulerCreator>(
    dur: Duration,
    scheduler_ctor: SchedulerCreator,
  ) -> Timeout<'a, Scheduler, Item>
  where
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    Timeout {
      dur,
      scheduler_ctor: FunctionWrapper::new(move |_| scheduler_ctor()),
      _item: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let scheduler_ctor = self.scheduler_ctor.clone();
    let dur = self.dur;
    Observable::create(move |s| {
      let sctl = StreamController::new(s);
      let timer = Arc::new(RwLock::new(None::<Subscription<'a>>));
      let scheduler_ctor = scheduler_ctor.clone();

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
            *timer.write().unwrap() = Some(
              observables::interval(dur, move || scheduler_ctor.call(()))
                .take(1)
                .subscribe(
                  move |_| {
                    sctl.sink_error(RxError::from_error(std::io::Error::from(
                      std::io::ErrorKind::TimedOut,
                    )));
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
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    Timeout::new(dur, scheduler_ctor).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let sbj = subjects::Subject::new();

    sbj
      .observable()
      .timeout(
        time::Duration::from_millis(100),
        schedulers::new_thread_scheduler(),
      )
      .subscribe(
        print_next_fmt!("{}"),
        |e| {
          if e.any_ref().is::<std::io::Error>() {
            println!("error - {:?}", e.cast_ref::<std::io::Error>());
          } else {
            println!("error - {:?}", e.any_ref());
          }
        },
        print_complete!(),
      );

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
