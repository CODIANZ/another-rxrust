use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SwitchOnNextOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  target: Observable<'a, Item>,
}

impl<'a, Item> SwitchOnNextOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(target: Observable<'a, Item>) -> SwitchOnNextOp<'a, Item> {
    SwitchOnNextOp { target }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let target = self.target.clone();
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);
      let emitted = Arc::new(RwLock::new(false));

      {
        let emitted = Arc::clone(&emitted);
        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        source.inner_subscribe(sctl.new_observer(
          move |serial, x| {
            if *emitted.read().unwrap() {
              sctl_next.upstream_abort_observe(&serial);
            } else {
              sctl_next.sink_next(x);
            }
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |serial| sctl_complete.sink_complete(&serial),
        ));
      };

      {
        let emitted = Arc::clone(&emitted);
        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();
        target.inner_subscribe(sctl.new_observer(
          move |_, x| {
            *emitted.write().unwrap() = true;
            sctl_next.sink_next(x);
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |_| sctl_complete.sink_complete_force(),
        ));
      }
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn switch_on_next(&self, target: Observable<'a, Item>) -> Observable<'a, Item> {
    SwitchOnNextOp::new(target).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = observables::interval(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    );
    let sbj = subjects::Subject::<u64>::new();

    o.switch_on_next(sbj.observable()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );

    thread::sleep(time::Duration::from_millis(1000));

    sbj.next(1000);
    sbj.next(2000);
    sbj.complete();
    thread::sleep(time::Duration::from_millis(1000));
  }
}
