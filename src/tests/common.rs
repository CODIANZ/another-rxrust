use crate::prelude::RxError;
use std::fmt::Display;
use std::sync::{Arc, RwLock};

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
  RxError::new(Box::new("any error".to_owned()))
}
#[cfg(not(feature = "anyhow"))]
pub fn error_to_string(e: &RxError) -> &String {
  e.error.downcast_ref::<String>().unwrap()
}

pub struct DestructChecker {
  memo: &'static str,
  n: usize,
  ref_count: Arc<RwLock<usize>>,
}
impl DestructChecker {
  pub fn new(memo: &'static str) -> DestructChecker {
    DestructChecker {
      memo,
      n: 0,
      ref_count: Arc::new(RwLock::new(0)),
    }
  }
}
impl Drop for DestructChecker {
  fn drop(&mut self) {
    if *self.ref_count.read().unwrap() == 0 {
      println!("destructed {}", self.memo);
    } else {
      *self.ref_count.write().unwrap() -= 1;
      println!("dropped {} #{}", self.memo, self.n);
    }
  }
}
impl Clone for DestructChecker {
  fn clone(&self) -> Self {
    *self.ref_count.write().unwrap() += 1;
    println!(
      "clone {} #{} -> #{}",
      self.memo,
      self.n,
      *self.ref_count.read().unwrap()
    );
    DestructChecker {
      memo: self.memo,
      n: *self.ref_count.read().unwrap(),
      ref_count: self.ref_count.clone(),
    }
  }
  fn clone_from(&mut self, source: &Self) {
    *self.ref_count.write().unwrap() += 1;
    self.memo = source.memo;
    self.n = *self.ref_count.read().unwrap();
    self.ref_count = source.ref_count.clone();
    println!("clone_from {} #{} -> #{}", source.memo, source.n, self.n);
  }
}
impl Display for DestructChecker {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("DestructChecker {} #{}", self.memo, self.n))
  }
}
