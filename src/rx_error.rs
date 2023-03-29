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
  pub error: Arc<Box<dyn std::any::Any + Send + Sync>>,
}
#[cfg(not(feature = "anyhow"))]
impl RxError {
  pub fn new(error: Box<dyn std::any::Any + Send + Sync>) -> RxError {
    RxError {
      error: Arc::new(error),
    }
  }
}
