use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SampleOp<'a, Item, TrigerValue>
where
  Item: Clone + Send + Sync,
  TrigerValue: Clone + Send + Sync,
{
  trigger: Observable<'a, TrigerValue>,
  _item: PhantomData<Item>,
}

impl<'a, Item, TrigerValue> SampleOp<'a, Item, TrigerValue>
where
  Item: Clone + Send + Sync,
  TrigerValue: Clone + Send + Sync,
{
  pub fn new(trigger: Observable<'a, TrigerValue>) -> SampleOp<'a, Item, TrigerValue>
  where
    TrigerValue: Clone + Send + Sync,
  {
    SampleOp {
      trigger,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let trigger = self.trigger.clone();
    Observable::<Item>::create(move |s| {
      let value = Arc::new(RwLock::new(None::<Item>));

      let sctl = StreamController::new(s);

      let value_trigger_next = Arc::clone(&value);
      let sctl_trigger_next = sctl.clone();

      trigger.inner_subscribe(sctl.new_observer(
        move |_, _| {
          let value = {
            let mut v = value_trigger_next.write().unwrap();
            let vv = v.clone();
            *v = None;
            vv
          };
          if let Some(v) = value {
            sctl_trigger_next.sink_next(v);
          }
        },
        |_, _| {},
        |_| {},
      ));

      let value_next = Arc::clone(&value);

      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          *value_next.write().unwrap() = Some(x);
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
    let sbj = subjects::Subject::new();
    observables::interval(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    )
    .sample(sbj.observable())
    .take(3)
    .subscribe(print_next_fmt!("{}"), print_error!(), print_complete!());

    (0..3).for_each(|_| {
      thread::sleep(time::Duration::from_millis(500));
      sbj.next(());
    });
    sbj.complete();
    thread::sleep(time::Duration::from_millis(500));
  }
}
