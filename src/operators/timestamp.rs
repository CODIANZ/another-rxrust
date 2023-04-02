use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{marker::PhantomData, time::SystemTime};

#[derive(Clone)]
pub struct TimestampOp<Item>
where
  Item: Clone + Send + Sync,
{
  _item: PhantomData<Item>,
}

impl<'a, Item> TimestampOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> TimestampOp<Item> {
    TimestampOp { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, (SystemTime, Item)> {
    Observable::create(move |s| {
      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next((SystemTime::now(), x));
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
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
  pub fn timestamp(&self) -> Observable<'a, (SystemTime, Item)> {
    TimestampOp::new().execute(self.clone())
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
    .timestamp()
    .take(10)
    .subscribe(print_next_fmt!("{:?}"), print_error!(), print_complete!());
    thread::sleep(time::Duration::from_millis(1500));
  }
}
