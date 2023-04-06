use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ReplaySubject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  subject: Arc<subject::Subject<'a, Item>>,
  items: Arc<RwLock<Vec<Item>>>,
  was_error: Arc<RwLock<Option<RxError>>>,
  was_completed: Arc<RwLock<bool>>,
}

impl<'a, Item> ReplaySubject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> ReplaySubject<'a, Item> {
    ReplaySubject {
      subject: Arc::new(subjects::Subject::new()),
      items: Arc::new(RwLock::new(Vec::new())),
      was_error: Arc::new(RwLock::new(None)),
      was_completed: Arc::new(RwLock::new(false)),
    }
  }

  pub fn next(&self, item: Item) {
    (*self.items.write().unwrap()).push(item.clone());
    self.subject.next(item);
  }
  pub fn error(&self, err: RxError) {
    *self.was_error.write().unwrap() = Some(err.clone());
    self.subject.error(err);
  }
  pub fn complete(&self) {
    *self.was_completed.write().unwrap() = true;
    self.subject.complete();
  }
  pub fn observable(&self) -> Observable<'a, Item> {
    let items = Arc::clone(&self.items);
    let was_error = Arc::clone(&self.was_error);
    let was_completed = Arc::clone(&self.was_completed);
    let subject = Arc::clone(&self.subject);

    Observable::create(move |s| {
      let is_continue = {
        // block until emitted for replay
        let items = &items.read().unwrap();
        let was_error = &*was_error.read().unwrap();
        let was_completed = &*was_completed.read().unwrap();
        items.iter().for_each(|x| {
          s.next(x.clone());
        });
        if let Some(err) = &*was_error {
          s.error(err.clone());
          false
        } else if *was_completed {
          s.complete();
          false
        } else {
          true
        }
      };
      if is_continue {
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
      }
    })
  }
}

#[cfg(test)]
mod tset {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let sbj = subjects::ReplaySubject::new();

    sbj.observable().subscribe(
      |x| println!("#1 next {}", x),
      |e| println!("#1 error {:?}", e),
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);
    sbj.complete();

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| println!("#2 error {:?}", e),
      || println!("#2 complete"),
    );
  }

  #[test]
  fn double() {
    let sbj = subjects::ReplaySubject::new();

    let binding = sbj.observable();
    let sbsc1 = binding.subscribe(
      |x| println!("#1 next {}", x),
      |e| {
        println!(
          "#1 error {:?}",
          e.downcast_ref::<&str>()
        )
      },
      || println!("#1 complete"),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);

    sbj.observable().subscribe(
      |x| println!("#2 next {}", x),
      |e| {
        println!(
          "#2 error {:?}",
          e.downcast_ref::<&str>()
        )
      },
      || println!("#2 complete"),
    );

    sbj.next(4);
    sbj.next(5);
    sbj.next(6);

    sbsc1.unsubscribe();

    sbj.next(7);
    sbj.next(8);
    sbj.next(9);

    sbj.error(RxError::from_error("ERR!"));

    sbj.observable().subscribe(
      |x| println!("#3 next {}", x),
      |e| {
        println!(
          "#3 error {:?}",
          e.downcast_ref::<&str>()
        )
      },
      || println!("#3 complete"),
    );
  }

  #[test]
  fn thread() {
    let sbj = subjects::ReplaySubject::new();

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
