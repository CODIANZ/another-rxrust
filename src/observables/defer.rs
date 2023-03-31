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
  use crate::tests::common::*;

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    let f = move || {
      let x = *n.read().unwrap();
      *n.write().unwrap() += 1;
      observables::just(x)
    };

    observables::defer(f.clone()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );

    observables::defer(f.clone()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
