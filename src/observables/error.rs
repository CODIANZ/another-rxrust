use crate::prelude::*;

pub fn error<'a, Item>(err: RxError) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::<Item>::create(move |s| {
    s.error(err.clone());
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::RxError;

  use super::error;

  #[cfg(feature = "anyhow")]
  use anyhow::anyhow;

  #[cfg(feature = "anyhow")]
  fn generate_error() -> RxError {
    RxError::new(anyhow!("anyhow error"))
  }

  #[cfg(not(feature = "anyhow"))]
  fn generate_error() -> RxError {
    RxError::new("string error".to_owned())
  }

  #[test]
  fn basic() {
    error::<String>(generate_error()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
  }
}
