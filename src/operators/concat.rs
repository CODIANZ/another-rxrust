use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  collections::VecDeque,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct Concat<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> Concat<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> Concat<'a, Item> {
    Concat { observables: observables.to_vec() }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let observables = Arc::new(RwLock::new(VecDeque::from_iter(
      self.observables.clone().into_iter(),
    )));
    Observable::create(move |s| {
      let sctl = StreamController::new(s);

      fn complete_and_next<'a, Item>(
        idx: usize,
        observables: Arc<RwLock<VecDeque<Observable<'a, Item>>>>,
        sctl: StreamController<'a, Item>,
      ) where
        Item: Clone + Send + Sync,
      {
        if observables.read().unwrap().is_empty() {
          sctl.sink_complete_force();
          return;
        }
        let o = observables.write().unwrap().pop_front().unwrap();

        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        o.inner_subscribe(sctl.new_observer(
          move |_, x| {
            sctl_next.sink_next(x);
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |_| {
            complete_and_next(
              idx + 1,
              Arc::clone(&observables),
              sctl_complete.clone(),
            );
          },
        ));
      }

      {
        let complete_and_next = complete_and_next.clone();
        let sctl_error = sctl.clone();
        let sctl_next = sctl.clone();
        let sctl_complete = sctl.clone();
        let observables = Arc::clone(&observables);
        source.inner_subscribe(sctl.new_observer(
          move |_, x| {
            sctl_next.sink_next(x);
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |_| {
            complete_and_next(
              0,
              Arc::clone(&observables),
              sctl_complete.clone(),
            );
          },
        ));
      }
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn concat(
    &self,
    observables: &[Observable<'a, Item>],
  ) -> Observable<'a, Item> {
    Concat::new(observables).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::from_iter(0..10)
      .observe_on(schedulers::new_thread_scheduler())
      .concat(&[
        observables::from_iter(10..20)
          .observe_on(schedulers::new_thread_scheduler()),
        observables::from_iter(20..30)
          .observe_on(schedulers::new_thread_scheduler()),
      ])
      .subscribe(
        print_next_fmt!("{:?}"),
        print_error!(),
        print_complete!(),
      );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
