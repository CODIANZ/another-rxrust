use std::marker::PhantomData;

use crate::internals::function_wrapper::*;
use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct RetryWhenOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  predicate_f: FunctionWrapper<'a, RxError, bool>,
  _item: PhantomData<Item>,
}

impl<'a, Item> RetryWhenOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> RetryWhenOp<'a, Item>
  where
    F: Fn(RxError) -> bool + Send + Sync + 'a,
  {
    RetryWhenOp {
      predicate_f: FunctionWrapper::new(f),
      _item: PhantomData,
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let f = self.predicate_f.clone();

    Observable::<Item>::create(move |s| {
      fn do_subscribe<'a, Item>(
        predicate: FunctionWrapper<'a, RxError, bool>,
        source: Observable<'a, Item>,
        sctl: StreamController<'a, Item>,
      ) where
        Item: Clone + Send + Sync,
      {
        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();
        let source_error = source.clone();
        source.inner_subscribe(sctl.new_observer(
          move |_, x: Item| {
            sctl_next.sink_next(x);
          },
          move |serial, e| {
            if predicate.call(e.clone()) {
              sctl_error.upstream_abort_observe(&serial);
              do_subscribe(predicate.clone(), source_error.clone(), sctl_error.clone());
            } else {
              sctl_error.sink_error(e);
            }
          },
          move |serial| sctl_complete.sink_complete(&serial),
        ));
      }

      let sctl = StreamController::new(s);
      do_subscribe(f.clone(), source.clone(), sctl.clone());
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn retry_when<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(RxError) -> bool + Send + Sync + 'a,
  {
    RetryWhenOp::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;
  use std::sync::{Arc, RwLock};

  #[test]
  fn basic() {
    let counter = Arc::new(RwLock::new(0));
    let counter_ob = Arc::clone(&counter);
    let o = Observable::create(move |s| {
      let c = *counter_ob.read().unwrap();
      println!("#{}", c + 1);
      s.next(c * 100 + 0);
      s.next(c * 100 + 1);
      *counter_ob.write().unwrap() += 1;
      if c < 5 {
        s.error(generate_error());
      } else {
        s.complete();
      }
    });

    o.retry_when(|e| {
      println!("retry_when {:}", error_to_string(&e));
      *counter.read().unwrap() < 2
    })
    .subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
