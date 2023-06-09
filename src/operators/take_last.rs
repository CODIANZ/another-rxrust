use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  collections::VecDeque,
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct TakeLast<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  _item: PhantomData<Item>,
}

impl<'a, Item> TakeLast<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> TakeLast<Item> {
    TakeLast { count, _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let count = self.count;

    Observable::<Item>::create(move |s| {
      let items = Arc::new(RwLock::new(VecDeque::new()));
      let items_next = Arc::clone(&items);
      let items_complete = Arc::clone(&items);
      let sctl = StreamController::new(s);
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          let mut items = items_next.write().unwrap();
          items.push_back(x);
          if items.len() > count {
            items.pop_front();
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          for x in items_complete.read().unwrap().iter() {
            if !sctl_complete.is_subscribed() {
              break;
            }
            sctl_complete.sink_next(x.clone());
          }
          sctl_complete.sink_complete(&serial)
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn take_last(&self, count: usize) -> Observable<'a, Item> {
    TakeLast::new(count).execute(self.clone())
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

    o.take_last(2).subscribe(
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

    o.take_last(2).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
