use crate::prelude::RxError;
use std::fmt::Debug;

#[derive(Clone)]
pub enum Material<T>
where
  T: Clone + Send + Sync,
{
  Next(T),
  Error(RxError),
  Complete,
}

impl<T> Debug for Material<T>
where
  T: Clone + Send + Sync,
  T: Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Self::Next(x) => f.write_str(&format!("Next({:?})", x)),
      Self::Error(x) => f.write_str(&format!("Error({:?})", x)),
      Self::Complete => f.write_str("Complete"),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    println!("{:?}", Material::Next(0));
    println!(
      "{:?}",
      Material::Error::<i32>(RxError::from_error("ERR!"))
    );
    println!("{:?}", Material::Complete::<i32>);
  }

  #[test]
  fn omit_scope() {
    use Material::*;
    println!("{:?}", Next(0));
    println!(
      "{:?}",
      Error::<i32>(RxError::from_error("ERR!"))
    );
    println!("{:?}", Complete::<i32>);
  }
}
