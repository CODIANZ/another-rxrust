use crate::prelude::*;
use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use super::function_wrapper::FunctionWrapper;

#[derive(Clone)]
pub struct StreamController<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  serial: Arc<RwLock<i32>>,
  subscriber: Observer<Item>,
  subscriptions: Arc<RwLock<HashMap<i32, Option<Subscription>>>>,
  on_finalize: Arc<RwLock<Vec<FunctionWrapper<(), ()>>>>,
}

impl<Item> StreamController<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn new(subscriber: Observer<Item>) -> StreamController<Item> {
    StreamController {
      serial: Arc::new(RwLock::new(0)),
      subscriber: subscriber,
      subscriptions: Arc::new(RwLock::new(HashMap::new())),
      on_finalize: Arc::new(RwLock::new(Vec::new())),
    }
  }

  pub fn set_on_finalize<F>(&self, f: F)
  where
    F: Fn() + Send + Sync + 'static,
  {
    self
      .on_finalize
      .write()
      .unwrap()
      .push(FunctionWrapper::new(move |_| f()));
  }

  pub fn upstream_prepare_serial(&self) -> i32 {
    let ret = {
      let mut x = self.serial.write().unwrap();
      let ret = *x;
      *x += 1;
      ret
    };

    self
      .subscriptions
      .write()
      .unwrap()
      .insert(ret.clone(), None);

    ret
  }

  pub fn sink_next(&self, x: Item) {
    self.subscriber.next(x);
  }

  pub fn sink_error(&self, e: RxError) {
    self.subscriber.error(e);
    self.finalize();
  }

  pub fn sink_complete(&self, serial: &i32) {
    let done_all = {
      let mut s = self.subscriptions.write().unwrap();
      s.remove(serial);
      s.len() == 0
    };
    if done_all {
      self.subscriber.complete();
      self.finalize();
    }
  }

  pub fn upstream_subscribe(&self, serial: &i32, sbsc: Subscription) {
    let mut s = self.subscriptions.write().unwrap();
    if s.contains_key(serial) {
      s.insert(serial.clone(), Some(sbsc));
    }
  }

  pub fn upstream_unsubscribe(&self, serial: &i32) {
    let mut s = self.subscriptions.write().unwrap();
    let o = s.remove(serial);
    if let Some(o) = o {
      if let Some(o) = o {
        o.unsubscribe();
      }
    }
  }

  pub fn finalize(&self) {
    self.subscriptions.read().unwrap().iter().for_each(|x| {
      if let Some(sbsc) = x.1 {
        sbsc.unsubscribe();
      }
    });
    self.subscriptions.write().unwrap().clear();
    self.subscriber.close();
    let mut handlers = self.on_finalize.write().unwrap();
    handlers.iter().for_each(|h| h.call(()));
    handlers.clear();
  }
}
