use crate::internals::function_wrapper::*;
use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Observer<'a, T>
where
  T: Clone + Send + Sync,
{
  fn_next: FunctionWrapper<'a, T, ()>,
  fn_error: FunctionWrapper<'a, RxError, ()>,
  fn_complete: FunctionWrapper<'a, (), ()>,
  fn_on_unsubscribe: Arc<RwLock<Option<FunctionWrapper<'a, (), ()>>>>,
}

impl<'a, T> Observer<'a, T>
where
  T: Clone + Send + Sync,
{
  pub fn new<Next, Error, Complete>(
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Observer<'a, T>
  where
    Next: Fn(T) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() -> () + Send + Sync + 'a,
  {
    Observer::<T> {
      fn_next: FunctionWrapper::new(next),
      fn_error: FunctionWrapper::new(error),
      fn_complete: FunctionWrapper::new(move |_| complete()),
      fn_on_unsubscribe: Arc::new(RwLock::new(None)),
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
  pub fn unsubscribe(&self) {
    self.fn_next.clear();
    self.fn_error.clear();
    self.fn_complete.clear();
    if let Some(f) = &*self.fn_on_unsubscribe.read().unwrap() {
      f.call(());
    }
    *self.fn_on_unsubscribe.write().unwrap() = None;
  }
  pub fn is_subscribed(&self) -> bool {
    self.fn_next.exists() && self.fn_error.exists() && self.fn_complete.exists()
  }
  pub(crate) fn set_on_unsubscribe<F>(&self, f: F)
  where
    F: Fn() -> () + Send + Sync + 'a,
  {
    *self.fn_on_unsubscribe.write().unwrap() =
      Some(FunctionWrapper::new(move |_| f()));
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::thread;

  #[test]
  fn basic() {
    let ob = Observer::new(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );
    ob.next(1);
    ob.next(2);
    ob.error(RxError::from_error("ERR!"));
    ob.complete();
  }

  #[test]
  fn basic_with_capture() {
    let gain = 100;
    let ob = Observer::new(
      move |x| println!("next {}", x + gain),
      print_error_as!(&str),
      print_complete!(),
    );
    ob.next(1);
    ob.next(2);
    ob.error(RxError::from_error("ERR!"));
    ob.complete();
  }

  #[test]
  fn close() {
    let ob = Observer::new(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    ob.next(1);
    ob.next(2);
    ob.unsubscribe();
  }

  #[test]
  fn clone_into_thread() {
    let ob = Observer::new(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
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
