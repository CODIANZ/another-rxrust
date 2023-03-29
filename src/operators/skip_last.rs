use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  collections::VecDeque,
  marker::PhantomData,
  sync::{Arc, RwLock},
};

pub struct SkipLastOp<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  _item: PhantomData<Item>,
}

impl<'a, Item> SkipLastOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> SkipLastOp<Item> {
    SkipLastOp {
      count,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let count = self.count;
    Observable::<Item>::create(move |s| {
      let items = Arc::new(RwLock::new(VecDeque::new()));
      let items_next = Arc::clone(&items);
      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          let emit = {
            let mut items = items_next.write().unwrap();
            items.push_back(x);
            if items.len() > count {
              items.pop_front()
            } else {
              None
            }
          };
          if let Some(x) = emit {
            sctl_next.sink_next(x);
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
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.skip_last(5).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        if !s.is_subscribed() {
          println!("break!");
          break;
        }
        println!("emit {}", n);
        s.next(n);
        thread::sleep(time::Duration::from_millis(100));
      }
      if s.is_subscribed() {
        s.complete();
      }
    });

    o.skip_last(2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
