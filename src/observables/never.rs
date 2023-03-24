use crate::prelude::*;

pub fn never<Item>() -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::<Item>::create(|_| Subscription::new(|| {}))
}

#[cfg(test)]
mod test {
  use super::never;

  #[test]
  fn basic() {
    never::<String>().subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e),
      || println!("complete"),
    );
  }
}
