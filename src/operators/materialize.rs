use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Materialize<Item>
where
  Item: Clone + Send + Sync,
{
  _item: PhantomData<Item>,
}

impl<'a, Item> Materialize<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> Materialize<Item> {
    Materialize { _item: PhantomData }
  }
  pub fn execute(
    &self,
    source: Observable<'a, Item>,
  ) -> Observable<'a, Material<Item>> {
    Observable::create(move |s| {
      let source = source.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(Material::Next(x));
        },
        move |serial, e| {
          sctl_error.sink_next(Material::Error(e));
          sctl_error.sink_complete(&serial);
        },
        move |serial| {
          sctl_complete.sink_next(Material::Complete);
          sctl_complete.sink_complete(&serial)
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn materialize(&self) -> Observable<'a, Material<Item>> {
    Materialize::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(0..10).materialize().subscribe(
      print_next_fmt!("{:?}"),
      print_error!(),
      print_complete!(),
    );
  }
}
