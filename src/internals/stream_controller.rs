use crate::prelude::*;
use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use super::function_wrapper::FunctionWrapper;

#[derive(Clone)]
pub struct StreamController<'a, Item>
where
  Item: Clone + Send + Sync + 'a,
{
  serial: Arc<RwLock<i32>>,
  subscriber: Observer<'a, Item>,
  unscribers: Arc<RwLock<HashMap<i32, FunctionWrapper<'a, (), ()>>>>,
  on_finalize: Arc<RwLock<Option<FunctionWrapper<'a, (), ()>>>>,
}

impl<'a, Item> StreamController<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(subscriber: Observer<Item>) -> StreamController<Item> {
    let subscriber_ = subscriber.clone();
    let sctl = StreamController {
      serial: Arc::new(RwLock::new(0)),
      subscriber,
      unscribers: Arc::new(RwLock::new(HashMap::new())),
      on_finalize: Arc::new(RwLock::new(None)),
    };
    {
      let sctl = sctl.clone();
      subscriber_.set_on_unsubscribe(move || sctl.finalize());
    }
    sctl
  }

  pub fn set_on_finalize<F>(&self, f: F)
  where
    F: Fn() + Send + Sync + 'a,
  {
    *self.on_finalize.write().unwrap() =
      Some(FunctionWrapper::new(move |_| f()));
  }

  pub fn new_observer<XItem, Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Observer<'a, XItem>
  where
    XItem: Clone + Send + Sync + 'a,
    Next: Fn(i32, XItem) + Send + Sync + 'a,
    Error: Fn(i32, RxError) + Send + Sync + 'a,
    Complete: Fn(i32) -> () + Send + Sync + 'a,
  {
    let serial = {
      let mut x = self.serial.write().unwrap();
      let ret = *x;
      *x += 1;
      ret
    };

    let serial_next = serial.clone();
    let serial_error = serial.clone();
    let serial_complete = serial.clone();
    let observer = Observer::new(
      move |x| next(serial_next, x),
      move |e| error(serial_error, e),
      move || complete(serial_complete),
    );
    let o_unsub = observer.clone();

    let mut unsubscribers = self.unscribers.write().unwrap();
    unsubscribers.insert(
      serial.clone(),
      FunctionWrapper::new(move |_| o_unsub.unsubscribe()),
    );
    observer
  }

  pub fn sink_next(&self, x: Item) {
    if self.subscriber.is_subscribed() {
      self.subscriber.next(x);
    } else {
      self.finalize();
    }
  }

  pub fn sink_error(&self, e: RxError) {
    if self.subscriber.is_subscribed() {
      self.subscriber.error(e);
      self.finalize();
    } else {
      self.finalize();
    }
  }

  pub fn sink_complete(&self, serial: &i32) {
    if self.subscriber.is_subscribed() {
      let done_all = {
        let mut observers = self.unscribers.write().unwrap();
        observers.remove(serial);
        observers.len() == 0
      };
      if done_all {
        self.subscriber.complete();
        self.finalize();
      }
    } else {
      self.finalize();
    }
  }

  pub fn sink_complete_force(&self) {
    if self.subscriber.is_subscribed() {
      self.subscriber.complete();
    }
    self.finalize();
  }

  pub fn upstream_abort_observe(&self, serial: &i32) {
    let mut observers = self.unscribers.write().unwrap();
    let o = observers.remove(serial);
    if let Some(o) = o {
      o.call(());
    }
  }

  pub fn finalize(&self) {
    self.unscribers.read().unwrap().iter().for_each(|x| {
      x.1.call(());
    });
    self.unscribers.write().unwrap().clear();
    if self.subscriber.is_subscribed() {
      self.subscriber.unsubscribe();
    }
    let on_finalize = &mut *self.on_finalize.write().unwrap();
    if let Some(f) = on_finalize {
      f.call(());
      *on_finalize = None;
    }
  }

  pub fn is_subscribed(&self) -> bool {
    self.subscriber.is_subscribed()
  }
}
