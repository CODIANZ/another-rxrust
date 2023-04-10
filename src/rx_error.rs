use std::{any::TypeId, sync::Arc};

struct RxErrorInner {
  error: Box<dyn std::any::Any + Send + Sync + 'static>,
  get_str: Box<dyn Fn(&Self) -> String + Send + Sync>,
}

#[derive(Clone)]
pub struct RxError {
  inner: Arc<RxErrorInner>,
}

impl RxError {
  pub fn from_error<E>(err: E) -> RxError
  where
    E: std::fmt::Debug + Send + Sync + 'static,
  {
    RxError {
      inner: Arc::new(RxErrorInner {
        error: Box::new(err),
        get_str: Box::new(|x: &RxErrorInner| {
          format!(
            "RxError({}) -> {:?}",
            std::any::type_name::<E>(),
            x.error.downcast_ref::<E>().unwrap()
          )
        }),
      }),
    }
  }

  pub fn from_result<T, E>(result: Result<T, E>) -> RxError
  where
    T: std::fmt::Debug,
    E: std::fmt::Debug + Send + Sync + 'static,
  {
    RxError {
      inner: Arc::new(RxErrorInner {
        error: Box::new(result.expect_err("Result must be Result::Err!")),
        get_str: Box::new(|x: &RxErrorInner| {
          format!(
            "RxError({}) -> {:?}",
            std::any::type_name::<E>(),
            x.error.downcast_ref::<E>().unwrap()
          )
        }),
      }),
    }
  }

  pub fn downcast_ref<E>(&self) -> Option<&E>
  where
    E: Send + Sync + 'static,
  {
    self.inner.error.downcast_ref::<E>()
  }

  pub fn type_id(&self) -> TypeId {
    self.inner.error.type_id()
  }

  pub fn is<T>(&self) -> bool
  where
    T: 'static,
  {
    self.inner.error.is::<T>()
  }
}

impl std::fmt::Debug for RxError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.pad(&self.to_string())
  }
}

impl std::string::ToString for RxError {
  fn to_string(&self) -> String {
    (self.inner.get_str)(&self.inner)
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use anyhow::anyhow;

  #[test]
  fn str_ref() {
    let error = Err::<i32, _>("aaa");

    let e = RxError::from_error(error.expect_err("?"));
    println!("{:?}", e.downcast_ref::<&str>());

    let e = RxError::from_result(error);
    println!("{:?}", e.downcast_ref::<&str>());
    println!("{:?}", e);
  }

  #[test]
  fn from_error() {
    let error = Err::<i32, _>(std::io::Error::from(
      std::io::ErrorKind::NotFound,
    ));
    let e = RxError::from_error(error.expect_err("?"));
    println!(
      "{:?}",
      e.downcast_ref::<std::io::Error>()
    );

    let error = Err::<i32, _>(std::io::Error::from(
      std::io::ErrorKind::NotFound,
    ));
    let e = RxError::from_result(error);
    println!(
      "{:?}",
      e.downcast_ref::<std::io::Error>()
    );
    println!("{:?}", e);
  }

  #[test]
  fn anyhow_error() {
    let error = Err::<i32, _>(anyhow!("anyhow error"));
    let e = RxError::from_result(error);
    println!(
      "{:?}",
      e.downcast_ref::<anyhow::Error>()
    );
    println!("{:?}", e);
  }
}
