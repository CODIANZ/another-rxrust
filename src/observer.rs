use crate::{internals::function_wrapper::FunctionWrapper, prelude::RxError};

#[derive(Clone)]
pub struct Observer<T>
where
  T: Clone + Send + Sync + 'static,
{
  fn_next: FunctionWrapper<T, ()>,
  fn_error: FunctionWrapper<RxError, ()>,
  fn_complete: FunctionWrapper<(), ()>,
}

impl<T> Observer<T>
where
  T: Clone + Send + Sync + 'static,
{
  pub fn new<Next, Error, Complete>(next: Next, error: Error, complete: Complete) -> Observer<T>
  where
    Next: Fn(T) + Send + Sync + 'static,
    Error: Fn(RxError) + Send + Sync + 'static,
    Complete: Fn() -> () + Send + Sync + 'static,
  {
    Observer::<T> {
      fn_next: FunctionWrapper::new(next),
      fn_error: FunctionWrapper::new(error),
      fn_complete: FunctionWrapper::new(move |_| complete()),
    }
  }
  pub fn next(&self, x: T) {
    self.fn_next.call_if_available(x);
  }
  pub fn error(&self, x: RxError) {
    self.fn_error.call_and_clear_if_available(x);
  }
  pub fn complete(&self) {
    self.fn_complete.call_and_clear_if_available(());
  }
  pub fn close(&self) {
    self.fn_next.clear();
    self.fn_error.clear();
    self.fn_complete.clear();
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::RxError;

  use super::Observer;
  use anyhow::anyhow;
  use std::thread;

  #[test]
  fn basic() {
    let ob = Observer::<i32>::new(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
    ob.next(1);
    ob.next(2);
    ob.error(RxError::new(anyhow!("abc")));
    ob.complete();
  }

  #[test]
  fn basic_with_capture() {
    let gain = 100;
    let ob = Observer::<i32>::new(
      move |x| println!("next {}", x + gain),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
    ob.next(1);
    ob.next(2);
    ob.error(RxError::new(anyhow!("abc")));
    ob.complete();
  }

  #[test]
  fn close() {
    let ob = Observer::<i32>::new(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
    ob.next(1);
    ob.next(2);
    ob.close();
  }

  #[test]
  fn clone_into_thread() {
    let ob = Observer::<i32>::new(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
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
