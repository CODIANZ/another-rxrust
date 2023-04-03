use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SkipWhile<'a, Item>
where
  Item: Clone + Send + Sync,
{
  predicate_f: FunctionWrapper<'a, Item, bool>,
}

impl<'a, Item> SkipWhile<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> SkipWhile<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    SkipWhile {
      predicate_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let f = self.predicate_f.clone();

    Observable::<Item>::create(move |s| {
      let enable = Arc::new(RwLock::new(false));

      let f = f.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          if *enable.read().unwrap() {
            sctl_next.sink_next(x);
          } else {
            if f.call(x.clone()) {
              sctl_next.sink_next(x);
              *enable.write().unwrap() = true;
            }
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn skip_while<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    SkipWhile::new(f).execute(self.clone())
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

    o.skip_while(|x| x > 5).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
