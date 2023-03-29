use crate::prelude::*;

pub fn never<'a, Item>() -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::<Item>::create(|_| {})
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::never::<String>().subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
