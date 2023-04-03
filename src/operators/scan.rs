use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ScanOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  reduce_f: FunctionWrapper<'a, (Item, Item), Item>,
}

impl<'a, Item> ScanOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> ScanOp<'a, Item>
  where
    F: Fn((Item, Item)) -> Item + Send + Sync + 'a,
  {
    ScanOp {
      reduce_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let f = self.reduce_f.clone();

    Observable::<Item>::create(move |s| {
      let f = f.clone();
      let result = Arc::new(RwLock::new(None::<Item>));

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let result_next = Arc::clone(&result);

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          {
            let mut r = result_next.write().unwrap();
            if let Some(xx) = &*r {
              *r = Some(f.call((xx.clone(), x)));
            } else {
              *r = Some(x);
            }
          }
          if let Some(x) = &*result_next.read().unwrap() {
            sctl_next.sink_next(x.clone());
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn scan<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn((Item, Item)) -> Item + Send + Sync + 'a,
  {
    ScanOp::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::range(1, 10).scan(|(a, b)| a + b).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn string() {
    observables::from_iter(["a".to_owned(), "b".to_owned(), "c".to_owned()].into_iter())
      .scan(|(a, b)| format!("{} - {}", a, b))
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", error_to_string(&e)),
        || println!("complete"),
      );
  }

  #[test]
  fn single() {
    observables::just(1).scan(|(a, b)| a + b).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn empty() {
    observables::empty::<i32>().scan(|(a, b)| a + b).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn error() {
    Observable::create(|s| {
      s.next(1);
      s.error(generate_error())
    })
    .scan(|(a, b)| a + b)
    .subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
