use crate::prelude::*;
use std::sync::Arc;

pub fn error<Item>(err: RxError) -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::<Item>::create(move |s| {
    s.error(Arc::clone(&err));
    Subscription::new(|| {})
  })
}

#[cfg(test)]
mod test {
  use super::error;
  use anyhow::anyhow;
  use std::sync::Arc;

  #[test]
  fn basic() {
    error::<String>(Arc::new(anyhow!("hoge"))).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e),
      || println!("complete"),
    );
  }
}
