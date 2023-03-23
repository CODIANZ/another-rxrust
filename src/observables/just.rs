// use crate::all::*;

// pub fn just<Item>(x: Item) -> Observable<Item>
// where
//   Item: Clone + 'static,
// {
//   Observable {
//     executor: rxfn(move |mut s, _| {
//       s.next(x.clone());
//       s.complete();
//     }),
//   }
// }
