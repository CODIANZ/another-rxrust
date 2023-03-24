use crate::all::*;

pub fn empty<Item>() -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::<Item>::create(|s, _| s.complete())
}

#[cfg(test)]
mod test {
  use crate::observable::IObservable;

  use super::empty;

  #[test]
  fn basic() {
    empty::<String>().subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", e),
      || println!("complete"),
    );
  }
}
