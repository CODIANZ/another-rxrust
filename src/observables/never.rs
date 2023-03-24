use crate::all::*;

pub fn never<Item>() -> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  Observable::<Item>::create(|_, _| {})
}

#[cfg(test)]
mod test {
  use crate::observable::IObservable;

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
