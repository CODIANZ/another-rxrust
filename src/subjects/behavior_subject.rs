use std::sync::{Arc, RwLock};

use crate::prelude::*;

#[derive(Clone)]
pub struct BehaviorSubject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  subject: Arc<subject::Subject<'a, Item>>,
  last_item: Arc<RwLock<Option<Item>>>,
  last_error: Arc<RwLock<Option<RxError>>>,
}

impl<'a, Item> BehaviorSubject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(initial: Item) -> BehaviorSubject<'a, Item> {
    BehaviorSubject {
      subject: Arc::new(subjects::Subject::new()),
      last_item: Arc::new(RwLock::new(Some(initial))),
      last_error: Arc::new(RwLock::new(None)),
    }
  }

  pub fn next(&self, item: Item) {
    *self.last_item.write().unwrap() = Some(item.clone());
    self.subject.next(item);
  }
  pub fn error(&self, err: RxError) {
    *self.last_error.write().unwrap() = Some(err.clone());
    self.subject.error(err);
  }
  pub fn complete(&self) {
    *self.last_item.write().unwrap() = None;
    self.subject.complete();
  }
  pub fn observable(&self) -> Observable<'a, Item> {
    let last_item = Arc::clone(&self.last_item);
    let last_error = Arc::clone(&self.last_error);
    let subject = Arc::clone(&self.subject);

    Observable::create(move |s| {
      {
        let last_item = &*last_item.read().unwrap();
        let last_error = &*last_error.read().unwrap();

        if let Some(err) = last_error {
          s.error(err.clone());
          return;
        }
        if let Some(item) = last_item {
          s.next(item.clone());
        } else {
          s.complete();
          return;
        }
      }
      let s_next = s.clone();
      let s_error = s.clone();
      let s_complete = s.clone();
      subject.observable().subscribe(
        move |x| s_next.next(x),
        move |e| s_error.error(e),
        move || {
          s_complete.complete();
        },
      );
    })
  }
}

#[cfg(test)]
mod tset {
  use super::BehaviorSubject;
  use crate::prelude::RxError;
  use std::{thread, time};

  #[cfg(feature = "anyhow")]
  use anyhow::anyhow;

  #[cfg(feature = "anyhow")]
  fn generate_error() -> RxError {
    RxError::new(anyhow!("anyhow error"))
  }

  #[cfg(not(feature = "anyhow"))]
  fn generate_error() -> RxError {
    RxError::new("string error".to_owned())
  }

  #[test]
  fn basic() {
    let sbj = BehaviorSubject::<i32>::new(100);

    sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", e.error),
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);
    sbj.complete();

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", e.error),
      || println!("#2 complete"),
    );
  }

  #[test]
  fn double() {
    let sbj = BehaviorSubject::<i32>::new(100);

    let binding = sbj.observable();
    let sbsc1 = binding.subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:}", e.error),
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", e.error),
      || println!("#2 complete"),
    );

    sbj.next(4);
    sbj.next(5);
    sbj.next(6);

    sbsc1.unsubscribe();

    sbj.next(7);
    sbj.next(8);
    sbj.next(9);

    sbj.error(generate_error());

    sbj.observable().subscribe(
      |x| println!("#3 next {}", x),
      |e| println!("#3 error {:}", e.error),
      || println!("#3 complete"),
    );
  }

  #[test]
  fn thread() {
    let sbj = BehaviorSubject::<i32>::new(100);

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
      |e| println!("#1 error {:}", e.error),
      || println!("#1 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:}", e.error),
      || println!("#2 complete"),
    );

    thread::sleep(time::Duration::from_millis(300));
    sbsc1.unsubscribe();

    th.join().ok();
  }
}
