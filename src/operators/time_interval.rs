use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
  time::{Duration, Instant},
};

#[derive(Clone)]
pub struct TimeIntervalOp<Item>
where
  Item: Clone + Send + Sync,
{
  _item: PhantomData<Item>,
}

impl<'a, Item> TimeIntervalOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> TimeIntervalOp<Item> {
    TimeIntervalOp { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Duration> {
    Observable::create(move |s| {
      let start_time = Arc::new(RwLock::new(None::<Instant>));
      let start_time_next = Arc::clone(&start_time);
      let start_time_complete = Arc::clone(&start_time);

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, _| {
          if let Some(start_time) = *start_time_next.read().unwrap() {
            sctl_next.sink_next(start_time.elapsed());
          }
          *start_time_next.write().unwrap() = Some(Instant::now());
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          if let Some(start_time) = *start_time_complete.read().unwrap() {
            sctl_complete.sink_next(start_time.elapsed());
          }
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn time_interval(&self) -> Observable<'a, Duration> {
    TimeIntervalOp::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::interval(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    )
    .time_interval()
    .take(10)
    .subscribe(print_next_fmt!("{:?}"), print_error!(), print_complete!());
    thread::sleep(time::Duration::from_millis(1500));
  }
}
