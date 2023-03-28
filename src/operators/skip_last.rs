use crate::{internals::stream_controller::StreamController, prelude::*};
use std::{
  collections::VecDeque,
  sync::{Arc, RwLock},
};

pub struct SkipLastOp<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  items: Arc<RwLock<VecDeque<Item>>>,
}

impl<'a, Item> SkipLastOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> SkipLastOp<Item> {
    SkipLastOp {
      count,
      items: Arc::new(RwLock::new(VecDeque::new())),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let count = self.count;
    let items = Arc::clone(&self.items);

    Observable::<Item>::create(move |s| {
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
      |e| println!("error {:}", e.error),
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
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
