use std::sync::Arc;

#[derive(Clone)]
pub struct RxError {
  error: Arc<Box<dyn std::any::Any + Send + Sync + 'static>>,
}

impl RxError {
  pub fn from_error<E>(err: E) -> RxError
  where
    E: std::fmt::Debug + Send + Sync + 'static,
  {
    RxError {
      error: Arc::new(Box::new(err)),
    }
  }

  pub fn from_result<T, E>(result: Result<T, E>) -> RxError
  where
    T: std::fmt::Debug,
    E: std::fmt::Debug + Send + Sync + 'static,
  {
    RxError {
      error: Arc::new(Box::new(result.expect_err("Result must be Result::Err!"))),
    }
  }

  pub fn cast_ref<E>(&self) -> &E
  where
    E: Send + Sync + 'static,
  {
    if let Some(x) = self.error.downcast_ref::<E>() {
      x
    } else {
      panic!("panic");
    }
  }

  pub fn any_ref(&self) -> &Box<dyn std::any::Any + Send + Sync + 'static> {
    self.error.as_ref()
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
    println!("{:?}", e.any_ref());
    println!("{:?}", e.cast_ref::<&str>());

    let e = RxError::from_result(error);
    println!("{:?}", e.any_ref());
    println!("{:?}", e.cast_ref::<&str>());
  }

  #[test]
  fn from_error() {
    let error = Err::<i32, _>(std::io::Error::from(std::io::ErrorKind::NotFound));
    let e = RxError::from_error(error.expect_err("?"));
    println!("{:?}", e.any_ref());
    println!("{:?}", e.cast_ref::<std::io::Error>());

    let error = Err::<i32, _>(std::io::Error::from(std::io::ErrorKind::NotFound));
    let e = RxError::from_result(error);
    println!("{:?}", e.any_ref());
    println!("{:?}", e.any_ref().downcast_ref::<std::io::Error>());
    println!("{:?}", e.cast_ref::<std::io::Error>());
  }

  #[test]
  #[should_panic]
  fn will_be_panic() {
    let error = Ok::<_, ()>(123);
    let e = RxError::from_result(error);
    println!("{:?}", e.any_ref());
    println!("{:?}", e.cast_ref::<anyhow::Error>());
  }

  #[test]
  fn anyhow_error() {
    let error = Err::<i32, _>(anyhow!("anyhow error"));
    let e = RxError::from_result(error);
    println!("{:?}", e.any_ref());
    println!("{:?}", e.cast_ref::<anyhow::Error>());
  }
}
