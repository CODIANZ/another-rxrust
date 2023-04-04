use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{marker::PhantomData, thread, time::Duration};

#[derive(Clone)]
pub struct Delay<Item>
where
  Item: Clone + Send + Sync,
{
  dur: Duration,
  _item: PhantomData<Item>,
}

impl<'a, Item> Delay<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(dur: Duration) -> Delay<Item> {
    Delay { dur, _item: PhantomData }
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
    Delay::new(dur).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
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
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
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
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
