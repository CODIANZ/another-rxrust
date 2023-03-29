use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

pub struct TakeUntilOp<'a, Item, TrigerValue>
where
  Item: Clone + Send + Sync,
  TrigerValue: Clone + Send + Sync,
{
  trigger: Observable<'a, TrigerValue>,
  _item: PhantomData<Item>,
}

impl<'a, Item, TrigerValue> TakeUntilOp<'a, Item, TrigerValue>
where
  Item: Clone + Send + Sync,
  TrigerValue: Clone + Send + Sync,
{
  pub fn new(trigger: Observable<'a, TrigerValue>) -> TakeUntilOp<'a, Item, TrigerValue>
  where
    TrigerValue: Clone + Send + Sync,
  {
    TakeUntilOp {
      trigger,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let trigger = self.trigger.clone();
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);

      let sctl_trigger_next = sctl.clone();
      trigger.inner_subscribe(sctl.new_observer(
        move |_, _| {
          sctl_trigger_next.sink_complete_force();
        },
        |_, _| {},
        |_| {},
      ));

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
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

    o.take_until(observables::timer(
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
