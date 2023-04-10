use crate::prelude::RxError;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Material<T>
where
  T: Clone + Send + Sync,
{
  Next(T),
  Error(RxError),
  Complete,
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

  #[test]
  fn custom_type() {
    use Material::*;
    #[derive(Clone)]
    struct X {}
    let x = Next(X {});
    // println!("{:?}", x); // compile error
    match x {
      Next(_) => println!("Next"),
      Error(_) => println!("Error"),
      Complete => println!("Complete"),
    }
  }
}
