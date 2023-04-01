use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{marker::PhantomData, thread, time::Duration};

#[derive(Clone)]
pub struct DelayOp<Item>
where
  Item: Clone + Send + Sync,
{
  dur: Duration,
  _item: PhantomData<Item>,
}

impl<'a, Item> DelayOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(dur: Duration) -> DelayOp<Item> {
    DelayOp {
      dur,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let dur = self.dur;

    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          thread::sleep(dur);
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
  pub fn delay(&self, dur: Duration) -> Observable<'a, Item> {
    DelayOp::new(dur).execute(self.clone())
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
      for n in 0..2 {
        println!("emit {}", n);
        s.next(n);
        thread::sleep(time::Duration::from_millis(500));
      }
      s.complete();
    });

    o.delay(time::Duration::from_millis(1000)).subscribe(
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

    o.skip(2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
