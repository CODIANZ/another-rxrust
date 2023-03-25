use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct FunctionWrapper<In, Out> {
  func: Arc<RwLock<Option<Box<dyn Fn(In) -> Out + Send + Sync + 'static>>>>,
}

impl<In, Out> FunctionWrapper<In, Out> {
  pub fn new<F>(func: F) -> FunctionWrapper<In, Out>
  where
    F: Fn(In) -> Out + Send + Sync + 'static,
  {
    FunctionWrapper {
      func: Arc::new(RwLock::new(Some(Box::new(func)))),
    }
  }
  pub fn clear(&self) {
    *self.func.write().unwrap() = None;
  }
  pub fn empty(&self) -> bool {
    self.func.read().unwrap().is_none()
  }
  pub fn call(&self, indata: In) -> Out {
    let f = self.func.read().unwrap();
    if let Some(ff) = &*f {
      ff(indata)
    } else {
      panic!("no func")
    }
  }
  pub fn call_if_available(&self, indata: In) -> Option<Out> {
    let f = self.func.read().unwrap();
    if let Some(ff) = &*f {
      Some(ff(indata))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tset {
  use super::FunctionWrapper;

  #[test]
  fn basic() {
    let f = FunctionWrapper::new(|x| x * x);
    assert_eq!(f.call(10), 100);
    assert_eq!(f.call(10), 100);
  }

  #[test]
  fn if_available() {
    let f = FunctionWrapper::new(|x| x * x);
    assert_eq!(f.call_if_available(10), Some(100));
    assert_eq!(f.empty(), false);
    f.clear();
    assert_eq!(f.empty(), true);
    assert_eq!(f.call_if_available(10), None);
  }
}
