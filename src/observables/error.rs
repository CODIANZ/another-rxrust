// use std::rc::Rc;

// use crate::all::*;

// pub fn error<Item>(e: RxError) -> Observable<Item>
// where
//   Item: Clone + 'static,
// {
//   Observable::<Item> {
//     executor: rxfn(move |mut s, _| {
//       let e = Rc::clone(&e);
//       s.error(e);
//     }),
//   }
// }
