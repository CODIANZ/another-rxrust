use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  ops::Add,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct SumAndCount<Item> {
  _item: PhantomData<Item>,
}

impl<'a, Item> SumAndCount<Item>
where
  Item: Clone + Send + Sync + Add<Output = Item>,
{
  pub fn new() -> SumAndCount<Item> {
    SumAndCount { _item: PhantomData }
  }
  pub fn execute(
    &self,
    source: Observable<'a, Item>,
  ) -> Observable<'a, (Item, usize)> {
    Observable::create(move |s| {
      let n = Arc::new(RwLock::new(0usize));
      let sum = Arc::new(RwLock::new(None::<Item>));

      let sctl = StreamController::new(s);
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let sum_next = Arc::clone(&sum);
      let n_next = Arc::clone(&n);

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          *n_next.write().unwrap() += 1;
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
            sctl_complete.sink_next((latest.clone(), *n.read().unwrap()));
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
  pub fn sum_and_count(&self) -> Observable<'a, (Item, usize)> {
    SumAndCount::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(1..10).sum_and_count().subscribe(
      print_next_fmt!("{:?}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn empty() {
    observables::empty::<i32>().sum_and_count().subscribe(
      print_next_fmt!("{:?}"),
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
    .sum_and_count()
    .subscribe(
      print_next_fmt!("{:?}"),
      print_error_as!(&str),
      print_complete!(),
    );
  }
}
