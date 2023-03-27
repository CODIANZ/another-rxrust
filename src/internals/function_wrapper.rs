use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct FunctionWrapperInner<'a, In, Out>
where
  In: Clone + Send + Sync + 'a,
  Out: Clone + Send + Sync + 'a,
{
  pub func: Arc<Box<dyn Fn(In) -> Out + Send + Sync + 'a>>,
}

#[derive(Clone)]
pub struct FunctionWrapper<'a, In, Out>
where
  In: Clone + Send + Sync + 'a,
  Out: Clone + Send + Sync + 'a,
{
  inner: Arc<RwLock<Option<FunctionWrapperInner<'a, In, Out>>>>,
}

impl<'a, In, Out> FunctionWrapper<'a, In, Out>
where
  In: Clone + Send + Sync + 'a,
  Out: Clone + Send + Sync + 'a,
{
  pub fn new<F>(func: F) -> FunctionWrapper<'a, In, Out>
  where
    F: Fn(In) -> Out + Send + Sync + 'a,
    In: Clone + Send + Sync + 'a,
    Out: Clone + Send + Sync + 'a,
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
