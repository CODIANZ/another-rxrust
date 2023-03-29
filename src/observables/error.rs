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
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::error::<String>(generate_error()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
