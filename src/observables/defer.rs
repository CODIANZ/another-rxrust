use crate::prelude::*;

pub fn defer<'a, Item, F>(f: F) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
  F: Fn() -> Observable<'a, Item> + Send + Sync + 'a,
{
  Observable::create(move |s| {
    f().inner_subscribe(s);
  })
}

#[cfg(test)]
mod test {
  use std::sync::{Arc, RwLock};

  use crate::prelude::*;

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    let f = move || {
      let x = *n.read().unwrap();
      *n.write().unwrap() += 1;
      observables::just(x)
    };

    observables::defer(f.clone()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );

    observables::defer(f.clone()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
