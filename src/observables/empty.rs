use crate::all::*;

pub fn empty<Item>() -> Observable<Item>
where
  Item: Clone + 'static,
{
  Observable::<Item> {
    executor: rxfn(|mut s, _| s.complete()),
  }
}
