use crate::prelude::RxError;

#[cfg(feature = "anyhow")]
pub fn generate_error() -> RxError {
  RxError::new(anyhow::anyhow!("anyhow error"))
}

#[cfg(feature = "anyhow")]
pub fn error_to_string(e: &RxError) -> String {
  format!("{:}", e.error)
}

#[cfg(not(feature = "anyhow"))]
pub fn generate_error() -> RxError {
  RxError::new(Box::new("any error"))
}
#[cfg(not(feature = "anyhow"))]
pub fn error_to_string(e: &RxError) -> &String {
  e.error.downcast_ref::<String>().unwrap()
}
