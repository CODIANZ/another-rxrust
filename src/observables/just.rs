use crate::prelude::*;

pub fn just<'a, Item>(x: Item) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::create(move |s| {
    s.next(x.clone());
    s.complete();
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::just("abc".to_owned()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
