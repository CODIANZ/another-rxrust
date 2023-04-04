use std::{any::TypeId, sync::Arc};

#[derive(Clone)]
pub struct RxError {
  error: Arc<Box<dyn std::any::Any + Send + Sync + 'static>>,
}

impl RxError {
  pub fn from_error<E>(err: E) -> RxError
  where
    E: std::fmt::Debug + Send + Sync + 'static,
  {
    RxError { error: Arc::new(Box::new(err)) }
  }

  pub fn from_result<T, E>(result: Result<T, E>) -> RxError
  where
    T: std::fmt::Debug,
    E: std::fmt::Debug + Send + Sync + 'static,
  {
    RxError {
      error: Arc::new(Box::new(
        result.expect_err("Result must be Result::Err!"),
      )),
    }
  }

  pub fn downcast_ref<E>(&self) -> Option<&E>
  where
    E: Send + Sync + 'static,
  {
    self.error.downcast_ref::<E>()
  }

  pub fn type_id(&self) -> TypeId {
    self.error.type_id()
  }

  pub fn is<T>(&self) -> bool
  where
    T: 'static,
  {
    self.error.is::<T>()
  }
}

impl std::fmt::Debug for RxError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RxError")
      .field("error", &format!("{:?}", self.error))
      .field("type_id", &self.error.type_id())
      .finish()
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
