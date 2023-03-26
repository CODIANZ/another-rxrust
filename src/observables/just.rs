use crate::prelude::*;

pub fn just<Item>(x: Item) -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::create(move |s| {
    s.next(x.clone());
    s.complete();
    Subscription::new(|| {})
  })
}

#[cfg(test)]
mod test {
  use super::just;

  #[test]
  fn basic() {
    just("abc".to_owned()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
  }
}
