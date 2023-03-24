use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use crate::all::*;

#[derive(Clone)]
pub struct Subject<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  observers: Arc<RwLock<HashMap<i32, Arc<Observer<Item>>>>>,
  serial: Arc<RwLock<i32>>,
}

impl<Item> Subject<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn new() -> Subject<Item> {
    Subject {
      observers: Arc::new(RwLock::new(HashMap::new())),
      serial: Arc::new(RwLock::new(0)),
    }
  }

  pub fn next(&self, item: Item) {
    let observers = self.observers.read().unwrap();
    for o in &*observers {
      o.1.next(item.clone());
    }
  }
  pub fn error(&self, err: RxError) {
    let mut observers = self.observers.write().unwrap();
    for o in &*observers {
      o.1.error(Arc::clone(&err));
    }
    observers.clear();
  }
  pub fn complete(&self) {
    let mut observers = self.observers.write().unwrap();
    for o in &*observers {
      o.1.complete();
    }
    observers.clear();
  }

  pub fn observable(&self) -> Observable<Item> {
    let observables = Arc::clone(&self.observers);
    let serial = Arc::clone(&self.serial);

    Observable::create(move |s| {
      let serial = {
        let mut serial = serial.write().unwrap();
        *serial += 1;
        *serial
      };
      {
        let mut observables = observables.write().unwrap();
        observables.insert(serial, Arc::clone(&s));
      }
      let observables = Arc::clone(&observables);
      Subscription::new(move || {
        observables.write().unwrap().remove(&serial);
      })
    })
  }
}

#[cfg(test)]
mod tset {
  use std::{thread, time};

  use crate::observable::IObservable;

  use super::Subject;

  #[test]
  fn basic() {
    let sbj = Subject::<i32>::new();

    sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", e),
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);
    sbj.complete();
  }

  #[test]
  fn double() {
    let sbj = Subject::<i32>::new();

    let sbsc1 = sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", e),
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", e),
      || println!("#2 complete"),
    );

    sbj.next(4);
    sbj.next(5);
    sbj.next(6);

    sbsc1.unsubscribe();

    sbj.next(7);
    sbj.next(8);
    sbj.next(9);

    sbj.complete();
  }

  #[test]
  fn thread() {
    let sbj = Subject::<i32>::new();

    let sbj_thread = sbj.clone();
    let th = thread::spawn(move || {
      for n in 0..10 {
        thread::sleep(time::Duration::from_millis(100));
        sbj_thread.next(n);
      }
      sbj_thread.complete();
    });

    let sbsc1 = sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", e),
      || println!("#1 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", e),
      || println!("#2 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));
    sbsc1.unsubscribe();

    th.join().ok();
  }
}
