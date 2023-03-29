use crate::{
  internals::{function_wrapper::FunctionWrapper, stream_controller::StreamController},
  prelude::*,
};

pub struct TakeWhileOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  predicate_f: FunctionWrapper<'a, Item, bool>,
}

impl<'a, Item> TakeWhileOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> TakeWhileOp<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    TakeWhileOp {
      predicate_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let f = self.predicate_f.clone();

    Observable::<Item>::create(move |s| {
      let f = f.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |serial, x: Item| {
          if f.call(x.clone()) {
            sctl_next.sink_next(x);
          } else {
            sctl_next.sink_complete(&serial)
          }
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
  use crate::tests::common::*;

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.take_while(|x| x < 5).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
