use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

pub struct SkipUntilOp<'a, Item, TrigerValue>
where
  Item: Clone + Send + Sync,
  TrigerValue: Clone + Send + Sync,
{
  trigger: Observable<'a, TrigerValue>,
  _item: PhantomData<Item>,
}

impl<'a, Item, TrigerValue> SkipUntilOp<'a, Item, TrigerValue>
where
  Item: Clone + Send + Sync,
  TrigerValue: Clone + Send + Sync,
{
  pub fn new(trigger: Observable<'a, TrigerValue>) -> SkipUntilOp<'a, Item, TrigerValue>
  where
    TrigerValue: Clone + Send + Sync,
  {
    SkipUntilOp {
      trigger,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let trigger = self.trigger.clone();
    Observable::<Item>::create(move |s| {
      let enable = Arc::new(RwLock::new(false));

      let sctl = StreamController::new(s);

      let enable_triggner_next = Arc::clone(&enable);
      let sctl_trigger_next = sctl.clone();
      trigger.inner_subscribe(sctl.new_observer(
        move |serial, _| {
          *enable_triggner_next.write().unwrap() = true;
          sctl_trigger_next.upstream_abort_observe(&serial);
        },
        |_, _| {},
        |_| {},
      ));

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          if *enable.read().unwrap() {
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
        thread::sleep(time::Duration::from_millis(100));
        s.next(n);
      }
      s.complete();
    });

    o.skip_until(observables::timer(
      time::Duration::from_millis(500),
      schedulers::new_thread_scheduler(),
    ))
    .subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
