use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct Count<Item> {
  _item: PhantomData<Item>,
}

impl<'a, Item> Count<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> Count<Item> {
    Count { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, usize> {
    Observable::<usize>::create(move |s| {
      let n = Arc::new(RwLock::new(0usize));

      let sctl = StreamController::new(s);
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let n_next = Arc::clone(&n);

      source.inner_subscribe(sctl.new_observer(
        move |_, _| {
          *n_next.write().unwrap() += 1;
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          let n = *n.read().unwrap();
          sctl_complete.sink_next(n);
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
  pub fn count(&self) -> Observable<'a, usize> {
    Count::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::repeat(()).take(100).count().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn empty() {
    observables::empty::<i32>().count().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error() {
    Observable::create(|s| {
      s.next(1);
      s.error(RxError::from_error("ERR!"))
    })
    .count()
    .subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
