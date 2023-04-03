use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct Last<Item>
where
  Item: Clone + Send + Sync,
{
  take_last_op: operators::TakeLast<Item>,
}

impl<'a, Item> Last<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> Last<Item> {
    Last {
      take_last_op: operators::TakeLast::new(1),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let take_last_op = self.take_last_op.clone();

    Observable::<Item>::create(move |s| {
      let source = source.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      take_last_op
        .execute(source)
        .inner_subscribe(sctl.new_observer(
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
  pub fn last(&self) -> Observable<'a, Item> {
    Last::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(0..10).last().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
