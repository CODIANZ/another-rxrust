use crate::prelude::*;

pub fn error<Item>(err: RxError) -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::<Item>::create(move |s| {
    s.error(err.clone());
    Subscription::new(|| {})
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
