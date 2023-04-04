use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct IgnoreElements<Item>
where
  Item: Clone + Send + Sync,
{
  _item: PhantomData<Item>,
}

impl<'a, Item> IgnoreElements<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> IgnoreElements<Item> {
    IgnoreElements { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, _| {},
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn ignore_elements(&self) -> Observable<'a, Item> {
    IgnoreElements::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(1..100).ignore_elements().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error() {
    observables::error::<()>(RxError::from_error("ERR!"))
      .ignore_elements()
      .subscribe(
        print_next!(),
        print_error!(),
        print_complete!(),
      );
  }
}
