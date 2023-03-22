use crate::all::*;

pub fn never<Item>() -> Observable<Item>
where
  Item: Clone + 'static,
{
  Observable::<Item> {
    executor: rxfn(|_, _| {}),
  }
}
