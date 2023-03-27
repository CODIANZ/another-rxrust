use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct FunctionWrapperInner<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  pub func: Arc<Box<dyn Fn(In) -> Out + Send + Sync + 'static>>,
}

#[derive(Clone)]
pub struct FunctionWrapper<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  inner: Arc<RwLock<Option<FunctionWrapperInner<In, Out>>>>,
}

impl<In, Out> FunctionWrapper<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  pub fn new<F>(func: F) -> FunctionWrapper<In, Out>
  where
    F: Fn(In) -> Out + Send + Sync + 'static,
    In: Clone + Send + Sync + 'static,
    Out: Clone + Send + Sync + 'static,
  {
    FunctionWrapper {
      inner: Arc::new(RwLock::new(Some(FunctionWrapperInner {
        func: Arc::new(Box::new(func)),
      }))),
    }
  }
  pub fn clear(&self) {
    *self.inner.write().unwrap() = None;
  }
  pub fn empty(&self) -> bool {
    self.inner.read().unwrap().is_none()
  }
  pub fn exists(&self) -> bool {
    !self.empty()
  }

  fn fetch_function(&self) -> Option<FunctionWrapperInner<In, Out>> {
    if let Some(x) = &*self.inner.read().unwrap() {
      Some(x.clone())
    } else {
      None
    }
  }

  pub fn call(&self, indata: In) -> Out {
    if let Some(ff) = self.fetch_function() {
      (ff.func)(indata)
    } else {
      panic!("no func")
    }
  }
  pub fn call_if_available(&self, indata: In) -> Option<Out> {
    if let Some(ff) = self.fetch_function() {
      Some((ff.func)(indata))
    } else {
      None
    }
  }
  pub fn call_and_clear_if_available(&self, indata: In) -> Option<Out> {
    let f = {
      let mut f = self.inner.write().unwrap();
      if let Some(x) = &*f {
        let x = x.clone();
        *f = None;
        Some(x)
      } else {
        None
      }
    };
    if let Some(f) = f {
      Some((f.func)(indata))
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
  fn call_if_available() {
    let f = FunctionWrapper::new(|x| x * x);
    assert_eq!(f.call_if_available(10), Some(100));
    assert_eq!(f.empty(), false);
    f.clear();
    assert_eq!(f.empty(), true);
    assert_eq!(f.call_if_available(10), None);
  }

  #[test]
  fn call_and_clear_if_available() {
    let f = FunctionWrapper::new(|x| x * x);
    assert_eq!(f.call_and_clear_if_available(10), Some(100));
    assert_eq!(f.empty(), true);
  }
}
