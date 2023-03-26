use std::sync::Arc;

#[derive(Clone)]
pub struct RxError {
  pub error: Arc<anyhow::Error>,
}

impl RxError {
  pub fn new(error: anyhow::Error) -> RxError {
    RxError {
      error: Arc::new(error),
    }
  }
}
