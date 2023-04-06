use crate::{internals::function_wrapper::FunctionWrapper, prelude::*};
use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct Subject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observers: Arc<RwLock<HashMap<i32, Observer<'a, Item>>>>,
  serial: Arc<RwLock<i32>>,
  on_subscribe: Arc<RwLock<Option<FunctionWrapper<'a, usize, ()>>>>,
  on_unsubscribe: Arc<RwLock<Option<FunctionWrapper<'a, usize, ()>>>>,
}

impl<'a, Item> Subject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> Subject<'a, Item> {
    Subject {
      observers: Arc::new(RwLock::new(HashMap::new())),
      serial: Arc::new(RwLock::new(0)),
      on_subscribe: Arc::new(RwLock::new(None)),
      on_unsubscribe: Arc::new(RwLock::new(None)),
    }
  }

  fn fetch_observers(&self) -> Vec<Observer<'a, Item>> {
    let binding = self.observers.read().unwrap();
    let x = binding.iter().map(|x| x.1.clone());
    Vec::from_iter(x)
  }

  pub fn next(&self, item: Item) {
    self
      .fetch_observers()
      .into_iter()
      .for_each(move |x| x.next(item.clone()));
  }
  pub fn error(&self, err: RxError) {
    let obs = self.fetch_observers();
    self.observers.write().unwrap().clear();
    obs.into_iter().for_each(move |x| x.error(err.clone()));
  }
  pub fn complete(&self) {
    let obs = self.fetch_observers();
    self.observers.write().unwrap().clear();
    obs.into_iter().for_each(|x| x.complete());
  }

  pub fn observable(&self) -> Observable<'a, Item> {
    let observers = Arc::clone(&self.observers);
    let serial = Arc::clone(&self.serial);

    let on_subscribe = Arc::clone(&self.on_subscribe);
    let on_unsubscribe = Arc::clone(&self.on_unsubscribe);

    Observable::create(move |s| {
      let serial = {
        let mut serial = serial.write().unwrap();
        *serial += 1;
        *serial
      };
      {
        let observers = Arc::clone(&observers);
        let on_unsubscribe = Arc::clone(&on_unsubscribe);
        s.set_on_unsubscribe(move || {
          let mut observers = observers.write().unwrap();
          observers.remove(&serial);
          if let Some(on_unsubscribe) = &*on_unsubscribe.read().unwrap() {
            on_unsubscribe.call(observers.len());
          }
        });
      }
      {
        let mut observers = observers.write().unwrap();
        observers.insert(serial, s);
        if let Some(on_subscribe) = &*on_subscribe.read().unwrap() {
          on_subscribe.call(observers.len());
        }
      }
    })
  }
  
  pub fn ref_count(&self) -> usize {
    self.observers.read().unwrap().len()
  }

  pub(crate) fn set_on_subscribe<F>(&self, f: F)
  where
    F: Fn(usize) + Send + Sync + 'a,
  {
    *self.on_subscribe.write().unwrap() = Some(FunctionWrapper::new(f));
  }

  pub(crate) fn set_on_unsubscribe<F>(&self, f: F)
  where
    F: Fn(usize) + Send + Sync + 'a,
  {
    *self.on_unsubscribe.write().unwrap() = Some(FunctionWrapper::new(f));
  }
}

#[cfg(test)]
mod tset {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let sbj = subjects::Subject::new();

    sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:?}", e),
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);
    sbj.complete();
  }

  #[test]
  fn double() {
    let sbj = subjects::Subject::new();
    println!("observers {}", sbj.ref_count());

    sbj.set_on_subscribe(|x| println!("on_subscribe {}", x));
    sbj.set_on_unsubscribe(|x| println!("on_unsubscribe {}", x));

    let binding = sbj.observable();
    let sbsc1 = binding.subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:?}", e),
      || println!("#1 complete"),
    );
    println!("observers {}", sbj.ref_count());

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:?}", e),
      || println!("#2 complete"),
    );
    println!("observers {}", sbj.ref_count());

    sbj.next(4);
    sbj.next(5);
    sbj.next(6);

    sbsc1.unsubscribe();
    println!("observers {}", sbj.ref_count());

    sbj.next(7);
    sbj.next(8);
    sbj.next(9);

    sbj.complete();
    println!("observers {}", sbj.ref_count());
  }

  #[test]
  fn thread() {
    let sbj = subjects::Subject::new();

    let sbj_thread = sbj.clone();
    let th = thread::spawn(move || {
      for n in 0..10 {
        thread::sleep(time::Duration::from_millis(100));
        sbj_thread.next(n);
      }
      sbj_thread.complete();
    });

    let binding = sbj.observable();
    let sbsc1 = binding.subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:?}", e),
      || println!("#1 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:?}", e),
      || println!("#2 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));
    sbsc1.unsubscribe();

    th.join().ok();
  }
}
