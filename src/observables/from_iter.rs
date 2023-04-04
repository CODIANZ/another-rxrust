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

  #[test]
  fn basic() {
    observables::from_iter(0..10).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn vec() {
    observables::from_iter(vec!["a", "b"].iter()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
