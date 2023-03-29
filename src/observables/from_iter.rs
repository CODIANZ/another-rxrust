use crate::prelude::*;

pub fn from_iter<'a, Iter, Item>(it: Iter) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
  Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
{
  Observable::create(move |s| {
    for x in it.clone() {
      if s.is_subscribed() {
        s.next(x);
      } else {
        break;
      }
    }
    if s.is_subscribed() {
      s.complete();
    }
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    observables::from_iter(0..10).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn vec() {
    observables::from_iter(vec!["a", "b"].iter()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
