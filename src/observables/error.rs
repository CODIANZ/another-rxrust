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
  use anyhow::anyhow;

  #[test]
  fn basic() {
    error::<String>(RxError::new(anyhow!("hoge"))).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
  }
}
