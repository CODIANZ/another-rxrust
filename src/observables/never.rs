use crate::prelude::*;

pub fn never<'a, Item>() -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::<Item>::create(|_| {})
}

#[cfg(test)]
mod test {
  use super::never;

  #[test]
  fn basic() {
    never::<String>().subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
  }
}
