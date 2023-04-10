use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Dematerialize<Item>
where
  Item: Clone + Send + Sync,
{
  _item: PhantomData<Item>,
}

impl<'a, Item> Dematerialize<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> Dematerialize<Item> {
    Dematerialize { _item: PhantomData }
  }
  pub fn execute(
    &self,
    source: Observable<'a, Material<Item>>,
  ) -> Observable<'a, Item> {
    Observable::create(move |s| {
      let source = source.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |serial, x: Material<Item>| match x {
          Material::Next(x) => sctl_next.sink_next(x),
          Material::Error(x) => sctl_next.sink_error(x),
          Material::Complete => sctl_next.sink_complete(&serial),
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Material<Item>>
where
  Item: Clone + Send + Sync,
{
  pub fn dematerialize(&self) -> Observable<'a, Item> {
    Dematerialize::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(0..10)
      .materialize()
      .take_last(3)
      .dematerialize()
      .subscribe(
        print_next_fmt!("{:?}"),
        print_error!(),
        print_complete!(),
      );
  }
}
