use crate::prelude::*;

pub fn just<'a, Item>(x: Item) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::create(move |s| {
    s.next(x.clone());
    s.complete();
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
