use std::sync::Arc;

#[cfg(feature = "anyhow")]
#[derive(Clone)]
pub struct RxError {
  pub error: Arc<anyhow::Error>,
}
#[cfg(feature = "anyhow")]
impl RxError {
  pub fn new(error: anyhow::Error) -> RxError {
    RxError {
      error: Arc::new(error),
    }
  }
}

#[cfg(not(feature = "anyhow"))]
#[derive(Clone)]
pub struct RxError {
  pub error: Arc<String>,
}
#[cfg(not(feature = "anyhow"))]
impl RxError {
  pub fn new(error: String) -> RxError {
    RxError {
      error: Arc::new(error),
    }
  }
}
