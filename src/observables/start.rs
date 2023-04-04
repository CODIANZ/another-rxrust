use crate::prelude::*;

pub fn start<'a, Item, F>(f: F) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
  F: Fn() -> Item + Send + Sync + 'a,
{
  Observable::create(move |s| {
    s.next(f());
    s.complete();
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::sync::{Arc, RwLock};

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    let f = move || {
      let x = *n.read().unwrap();
      *n.write().unwrap() += 1;
      x
    };

    observables::start(f.clone()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );

    observables::start(f.clone()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
