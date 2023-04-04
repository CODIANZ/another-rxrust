use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  ops::Add,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct Sum<Item> {
  _item: PhantomData<Item>,
}

impl<'a, Item> Sum<Item>
where
  Item: Clone + Send + Sync + Add<Output = Item>,
{
  pub fn new() -> Sum<Item> {
    Sum { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    Observable::<Item>::create(move |s| {
      let sum = Arc::new(RwLock::new(None::<Item>));

      let sctl = StreamController::new(s);
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let sum_next = Arc::clone(&sum);

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          let mut sum = sum_next.write().unwrap();
          if let Some(latest) = &*sum {
            *sum = Some(latest.clone() + x);
          } else {
            *sum = Some(x);
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          if let Some(latest) = &*sum.read().unwrap() {
            sctl_complete.sink_next(latest.clone());
          }
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + Add<Output = Item>,
{
  pub fn sum(&self) -> Observable<'a, Item> {
    Sum::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(1..10).sum().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn empty() {
    observables::empty::<i32>().sum().subscribe(
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
    .sum()
    .subscribe(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );
  }
}
