use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct FirstOp<Item>
where
  Item: Clone + Send + Sync,
{
  take_op: operators::TakeOp<Item>,
}

impl<'a, Item> FirstOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> FirstOp<Item> {
    FirstOp {
      take_op: operators::TakeOp::new(1),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let take_op = self.take_op.clone();

    Observable::<Item>::create(move |s| {
      let source = source.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      take_op.execute(source).inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(x);
        },
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
  pub fn first(&self) -> Observable<'a, Item> {
    operators::FirstOp::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(0..10).first().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
