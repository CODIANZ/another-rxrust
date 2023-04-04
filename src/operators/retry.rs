use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Retry<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  _item: PhantomData<Item>,
}

impl<'a, Item> Retry<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> Retry<Item> {
    Retry { count, _item: PhantomData }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let count = self.count;

    Observable::<Item>::create(move |s| {
      fn do_subscribe<'a, Item>(
        n: usize,
        max_retry: usize,
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
            if max_retry == 0 || n < max_retry {
              sctl_error.upstream_abort_observe(&serial);
              do_subscribe(
                n + 1,
                max_retry,
                source_error.clone(),
                sctl_error.clone(),
              );
            } else {
              sctl_error.sink_error(e);
            }
          },
          move |serial| sctl_complete.sink_complete(&serial),
        ));
      }

      let sctl = StreamController::new(s);
      do_subscribe(1, count, source.clone(), sctl.clone());
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn retry(&self, max_retry: usize) -> Observable<'a, Item> {
    Retry::new(max_retry).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
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
        s.error(RxError::from_error("ERR!"));
      } else {
        s.complete();
      }
    });

    o.retry(0).subscribe(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );

    *counter.write().unwrap() = 0;
    o.retry(3).subscribe(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );
  }
}
