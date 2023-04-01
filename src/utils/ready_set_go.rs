use crate::prelude::*;

pub fn ready_set_go<'a, F, Item>(f: F, o: Observable<'a, Item>) -> Observable<'a, Item>
where
  F: Fn() + Send + Sync + 'a,
  Item: Clone + Send + Sync,
{
  Observable::create(move |s| {
    o.inner_subscribe(s);
    f();
  })
}
#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::error_to_string;

  #[test]
  fn basic() {
    let s = subjects::Subject::<i32>::new();
    let s_f = s.clone();
    utils::ready_set_go(
      move || {
        s_f.next(1);
        s_f.next(2);
        s_f.complete();
      },
      s.observable(),
    )
    .subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
