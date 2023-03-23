use crate::all::RxError;
use std::sync::RwLock;

type FnNext<T> = Box<dyn Fn(T) + Send + Sync>;
type FnError = Box<dyn Fn(RxError) + Send + Sync>;
type FnComplete = Box<dyn Fn() + Send + Sync>;

pub struct Observer<T> {
  fn_next: FnNext<T>,
  fn_error: FnError,
  fn_complete: FnComplete,
  closed: RwLock<bool>,
}

impl<T> Observer<T> {
  pub fn new<Next, Error, Complete>(next: Next, error: Error, complete: Complete) -> Observer<T>
  where
    Next: Fn(T) + Send + Sync + 'static,
    Error: Fn(RxError) + Send + Sync + 'static,
    Complete: Fn() + Send + Sync + 'static,
  {
    Observer::<T> {
      fn_next: Box::new(next),
      fn_error: Box::new(error),
      fn_complete: Box::new(complete),
      closed: RwLock::new(false),
    }
  }
  pub fn next(&self, x: T) {
    if !self.is_closed() {
      (self.fn_next)(x);
    }
  }
  pub fn error(&self, x: RxError) {
    if !self.close() {
      (self.fn_error)(x);
    }
  }
  pub fn complete(&self) {
    if !self.close() {
      (self.fn_complete)();
    }
  }
  pub fn is_closed(&self) -> bool {
    *self.closed.read().unwrap()
  }
  pub fn close(&self) -> bool {
    let mut old_closed = false;
    if let Ok(mut closed) = self.closed.write() {
      old_closed = *closed;
      *closed = true;
    }
    old_closed
  }
}

mod test {
  use super::Observer;
  use anyhow::anyhow;
  use std::{sync::Arc, thread};

  #[test]
  fn basic() {
    let ob = Observer::<i32>::new(
      |x| println!("next {}", x),
      |e| println!("{:}", e),
      || println!("complete"),
    );
    ob.next(1);
    ob.next(2);
    ob.error(Arc::new(anyhow!("abc")));
    ob.complete();
  }

  #[test]
  fn basic_with_capture() {
    let gain = 100;
    let ob = Observer::<i32>::new(
      move |x| println!("next {}", x + gain),
      |e| println!("{:}", e),
      || println!("complete"),
    );
    ob.next(1);
    ob.next(2);
    ob.error(Arc::new(anyhow!("abc")));
    ob.complete();
  }

  #[test]
  fn close() {
    let ob = Observer::<i32>::new(
      |x| println!("next {}", x),
      |e| println!("{:}", e),
      || println!("complete"),
    );
    ob.next(1);
    ob.next(2);
    ob.close();
    assert_eq!(ob.is_closed(), true);
  }

  #[test]
  fn clone_into_thread() {
    let ob = Arc::new(Observer::<i32>::new(
      |x| println!("next {}", x),
      |e| println!("{:}", e),
      || println!("complete"),
    ));
    let a = ob.clone();
    let b = ob.clone();
    let th_a = thread::spawn(move || {
      for n in 0..100 {
        a.next(n);
      }
      a.complete();
    });
    let th_b = thread::spawn(move || {
      for n in 0..10 {
        b.next(100 + n);
      }
      b.complete();
    });

    th_a.join().ok();
    th_b.join().ok();
  }
}
