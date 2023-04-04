use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use scheduler::IScheduler;
use std::sync::{Arc, RwLock};
use std::{marker::PhantomData, thread, time::Duration};

#[derive(Clone)]
pub struct Debounce<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  dur: Duration,
  scheduler_ctor: FunctionWrapper<'a, (), Scheduler>,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Scheduler, Item> Debounce<'a, Scheduler, Item>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync,
  Item: Clone + Send + Sync,
{
  pub fn new<SchedulerCreator>(
    dur: Duration,
    scheduler_ctor: SchedulerCreator,
  ) -> Debounce<'a, Scheduler, Item>
  where
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    Debounce {
      dur,
      scheduler_ctor: FunctionWrapper::new(move |_| scheduler_ctor()),
      _item: PhantomData,
      _lifetime: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let dur = self.dur;
    let scheduler_ctor = self.scheduler_ctor.clone();

    Observable::<Item>::create(move |s| {
      let value = Arc::new(RwLock::new(None::<Item>));

      let sctl = StreamController::new(s);

      let scheduler = scheduler_ctor.call(());
      {
        let scheduler = scheduler.clone();
        sctl.set_on_finalize(move || {
          scheduler.abort();
        });
      }
      {
        let sctl = sctl.clone();
        let value = Arc::clone(&value);
        scheduler.post(move || {
          while sctl.is_subscribed() {
            thread::sleep(dur);
            let value = {
              let mut value = value.write().unwrap();
              let v = value.clone();
              *value = None;
              v
            };
            if let Some(value) = value {
              sctl.sink_next(value);
            }
          }
        })
      }

      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          *value.write().unwrap() = Some(x);
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
  pub fn debounce<Scheduler, SchedulerCreator>(
    &self,
    dur: Duration,
    scheduler_ctor: SchedulerCreator,
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    Debounce::new(dur, scheduler_ctor).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::interval(
      time::Duration::from_millis(10),
      schedulers::new_thread_scheduler(),
    )
    .debounce(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    )
    .take(5)
    .subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );

    thread::sleep(time::Duration::from_millis(1000));
  }
}
