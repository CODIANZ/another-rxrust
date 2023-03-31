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
  use std::sync::{Arc, RwLock};

  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    let f = move || {
      let x = *n.read().unwrap();
      *n.write().unwrap() += 1;
      x
    };

    observables::start(f.clone()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );

    observables::start(f.clone()).subscribe(
      |x| println!("next {}", x),
      |e| println!("{:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
