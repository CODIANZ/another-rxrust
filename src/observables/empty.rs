use crate::prelude::*;

pub fn empty<'a, Item>() -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::<Item>::create(|s| {
    s.complete();
  })
}

#[cfg(test)]
mod test {
  use super::empty;

  #[test]
  fn basic() {
    empty::<String>().subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
  }
}
