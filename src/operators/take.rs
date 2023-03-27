use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

use crate::{internals::stream_controller::StreamController, prelude::*};

pub struct TakeOp<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  _item: PhantomData<Item>,
}

impl<'a, Item> TakeOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> TakeOp<Item> {
    TakeOp {
      count,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let count = self.count;

    Observable::<Item>::create(move |s| {
      let n = Arc::new(RwLock::new(0));

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |serial, x| {
          let (emit, complete) = {
            let mut n = n.write().unwrap();
            let nn = *n;
            *n += 1;
            (nn < count, (nn + 1) >= count)
          };
          if emit {
            sctl_next.sink_next(x);
          }
          if complete {
            sctl_next.upstream_abort_observe(&serial);
            sctl_next.sink_complete(&serial);
            sctl_next.finalize();
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

    o.take(2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::create(|s| {
      for n in 0..100 {
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

    o.take(2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
