use crate::prelude::*;
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
}

impl<'a, Item> Subject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> Subject<'a, Item> {
    Subject {
      observers: Arc::new(RwLock::new(HashMap::new())),
      serial: Arc::new(RwLock::new(0)),
    }
  }

  pub fn next(&self, item: Item) {
    self.observers.read().unwrap().iter().for_each(|x| {
      x.1.next(item.clone());
    });
  }
  pub fn error(&self, err: RxError) {
    self.observers.read().unwrap().iter().for_each(|x| {
      x.1.error(err.clone());
    });
    self.observers.write().unwrap().clear();
  }
  pub fn complete(&self) {
    self.observers.read().unwrap().iter().for_each(|x| {
      x.1.complete();
    });
    self.observers.write().unwrap().clear();
  }

  pub fn observable(&self) -> Observable<'a, Item> {
    let observers = Arc::clone(&self.observers);
    let serial = Arc::clone(&self.serial);

    Observable::create(move |s| {
      let serial = {
        let mut serial = serial.write().unwrap();
        *serial += 1;
        *serial
      };
      {
        let observers = observers.clone();
        s.set_on_unsubscribe(move || {
          observers.write().unwrap().remove(&serial);
        });
      }
      {
        let mut observers = observers.write().unwrap();
        observers.insert(serial, s);
      }
    })
  }
  pub fn ref_count(&self) -> usize {
    self.observers.read().unwrap().len()
  }
}

#[cfg(test)]
mod tset {
  use crate::prelude::*;
  use crate::tests::common::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let sbj = subjects::Subject::new();

    sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", error_to_string(&e)),
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

    let binding = sbj.observable();
    let sbsc1 = binding.subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", error_to_string(&e)),
      || println!("#1 complete"),
    );
    println!("observers {}", sbj.ref_count());

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", error_to_string(&e)),
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
      |e| println!("#1 error {:}", error_to_string(&e)),
      || println!("#1 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", error_to_string(&e)),
      || println!("#2 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));
    sbsc1.unsubscribe();

    th.join().ok();
  }
}
