use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct ElementAtOp<Item>
where
  Item: Clone + Send + Sync,
{
  take_op: operators::TakeOp<Item>,
}

impl<'a, Item> ElementAtOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> ElementAtOp<Item> {
    ElementAtOp {
      take_op: operators::TakeOp::<Item>::new(count),
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

      take_op
        .execute(source)
        .last()
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

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_iter(1..100)
      .element_at(86)
      .tap(|x| assert_eq!(x, 86), junk_error!(), junk_complete!())
      .subscribe(print_next_fmt!("{}"), print_error!(), print_complete!());
  }
}
