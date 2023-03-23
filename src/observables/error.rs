use crate::all::*;
use std::sync::Arc;

pub fn error<Item>(err: RxError) -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::create(move |s, _| s.error(Arc::clone(&err)))
}

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
