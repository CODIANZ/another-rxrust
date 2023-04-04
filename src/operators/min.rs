use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct Min<Item> {
  _item: PhantomData<Item>,
}

impl<'a, Item> Min<Item>
where
  Item: Clone + Send + Sync + PartialOrd,
{
  pub fn new() -> Min<Item> {
    Min { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    Observable::<Item>::create(move |s| {
      let result = Arc::new(RwLock::new(None::<Item>));

      let sctl = StreamController::new(s);
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let result_next = Arc::clone(&result);

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          let mut r = result_next.write().unwrap();
          if let Some(xx) = &*r {
            if x < *xx {
              *r = Some(x);
            }
          } else {
            *r = Some(x);
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          if let Some(x) = &*result.read().unwrap() {
            sctl_complete.sink_next(x.clone());
          }
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + PartialOrd,
{
  pub fn min(&self) -> Observable<'a, Item> {
    Min::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter([5, 6, 2, 7].into_iter())
      .min()
      .subscribe(
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );
  }

  #[test]
  fn empty() {
    observables::empty::<i32>().min().subscribe(
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
    .min()
    .subscribe(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );
  }
}
