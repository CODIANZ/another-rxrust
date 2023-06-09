use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct SkipUntil<'a, Item, TriggerValue>
where
  Item: Clone + Send + Sync,
  TriggerValue: Clone + Send + Sync,
{
  trigger: Observable<'a, TriggerValue>,
  _item: PhantomData<Item>,
}

impl<'a, Item, TriggerValue> SkipUntil<'a, Item, TriggerValue>
where
  Item: Clone + Send + Sync,
  TriggerValue: Clone + Send + Sync,
{
  pub fn new(
    trigger: Observable<'a, TriggerValue>,
  ) -> SkipUntil<'a, Item, TriggerValue>
  where
    TriggerValue: Clone + Send + Sync,
  {
    SkipUntil { trigger, _item: PhantomData }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let trigger = self.trigger.clone();
    Observable::<Item>::create(move |s| {
      let enable = Arc::new(RwLock::new(false));
      let sctl = StreamController::new(s);

      let obs_trigger = {
        let enable_triggner_next = Arc::clone(&enable);
        let sctl_trigger_next = sctl.clone();

        sctl.new_observer(
          move |serial, _| {
            *enable_triggner_next.write().unwrap() = true;
            sctl_trigger_next.upstream_abort_observe(&serial);
          },
          |_, _| {},
          |_| {},
        )
      };

      let obs_source = {
        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        sctl.new_observer(
          move |_, x| {
            if *enable.read().unwrap() {
              sctl_next.sink_next(x);
            }
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |_| sctl_complete.sink_complete_force(), // trigger also unsubscribe
        )
      };

      trigger.inner_subscribe(obs_trigger);
      source.inner_subscribe(obs_source);
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn skip_until<TriggerValue>(
    &self,
    trigger: Observable<'a, TriggerValue>,
  ) -> Observable<'a, Item>
  where
    TriggerValue: Clone + Send + Sync,
  {
    SkipUntil::new(trigger).execute(self.clone())
  }
}

#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
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
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
