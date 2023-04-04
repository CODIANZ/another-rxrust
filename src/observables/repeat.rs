use crate::prelude::*;

pub fn repeat<'a, Item>(x: Item) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::create(move |s| {
    while s.is_subscribed() {
      s.next(x.clone());
    }
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::repeat("abc".to_owned()).take(5).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
