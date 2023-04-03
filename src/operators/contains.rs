use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct Contains<Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  target: Item,
}

impl<'a, Item> Contains<Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn new(target: Item) -> Contains<Item> {
    Contains { target }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, bool> {
    let target = self.target.clone();

    Observable::<bool>::create(move |s| {
      let target = target.clone();
      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();
      source.inner_subscribe(sctl.new_observer(
        move |serial, x| {
          if x == target {
            sctl_next.sink_next(true);
            sctl_next.sink_complete(&serial);
          }
        },
        move |serial, _| {
          sctl_error.sink_next(false);
          sctl_error.sink_complete(&serial);
        },
        move |serial| {
          sctl_complete.sink_next(false);
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn contains(&self, target: Item) -> Observable<'a, bool> {
    Contains::new(target).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::from_iter(0..10).contains(5).subscribe(
      |x| {
        println!("{}", x);
        assert_eq!(x, true);
      },
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn not_found() {
    observables::from_iter(0..10).contains(100).subscribe(
      |x| {
        println!("{}", x);
        assert_eq!(x, false);
      },
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error() {
    observables::error(generate_error()).contains(5).subscribe(
      |x| {
        println!("{}", x);
        assert_eq!(x, false);
      },
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn empty() {
    observables::empty().contains(5).subscribe(
      |x| {
        println!("{}", x);
        assert_eq!(x, false);
      },
      print_error!(),
      print_complete!(),
    );
  }
}
