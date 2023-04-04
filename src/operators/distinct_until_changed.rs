use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct DistinctUntilChanged<Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  _item: PhantomData<Item>,
}

impl<'a, Item> DistinctUntilChanged<Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn new() -> DistinctUntilChanged<Item> {
    DistinctUntilChanged { _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    Observable::<Item>::create(move |s| {
      let last = Arc::new(RwLock::new(Option::<Item>::None));

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          let last_x = {
            if let Some(x) = &*last.read().unwrap() {
              Some(x.clone())
            } else {
              None
            }
          };
          if let Some(last_x) = last_x {
            if last_x != x {
              *last.write().unwrap() = Some(x.clone());
              sctl_next.sink_next(x);
            }
          } else {
            *last.write().unwrap() = Some(x.clone());
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn distinct_until_changed(&self) -> Observable<'a, Item> {
    DistinctUntilChanged::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      s.next(0);
      s.next(0);
      s.next(1);
      s.next(1);
      s.next(2);
      s.next(2);
      s.next(2);
      s.next(2);
      s.next(3);
      s.complete();
    });

    o.distinct_until_changed().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
