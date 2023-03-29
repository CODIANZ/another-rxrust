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
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::just("abc".to_owned()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
