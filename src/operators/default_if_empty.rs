use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::sync::{Arc, RwLock};

pub struct DefaultIfEmpty<Item>
where
  Item: Clone + Send + Sync,
{
  default: Item,
}

impl<'a, Item> DefaultIfEmpty<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(default: Item) -> DefaultIfEmpty<Item> {
    DefaultIfEmpty { default }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let default = self.default.clone();
    let emitted = Arc::new(RwLock::new(false));

    Observable::<Item>::create(move |s| {
      let default_complete = default.clone();

      let emitted_next = Arc::clone(&emitted);
      let emitted_complete = Arc::clone(&emitted);

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();
      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          *emitted_next.write().unwrap() = true;
          sctl_next.sink_next(x);
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          if !*emitted_complete.read().unwrap() {
            sctl_complete.sink_next(default_complete.clone());
          }
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::empty().default_if_empty(5).subscribe(
      |x| {
        println!("{}", x);
        assert_eq!(x, 5);
      },
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn not_empty() {
    observables::from_iter(0..10)
      .default_if_empty(100)
      .subscribe(
        |x| {
          println!("{}", x);
          assert_ne!(x, 100);
        },
        print_error!(),
        print_complete!(),
      );
  }

  #[test]
  fn error() {
    observables::error(generate_error())
      .default_if_empty(5)
      .subscribe(
        |_| assert!(true, "don't come here"),
        print_error!(),
        print_complete!(),
      );
  }
}
