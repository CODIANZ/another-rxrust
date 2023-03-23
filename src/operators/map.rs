// use std::{cell::RefCell, rc::Rc};

// use crate::all::*;

// pub fn map<In, Out>(src: Observable<In>, f: RxFn<dyn FnMut(In) -> Out>) -> Observable<Out>
// where
//   Out: Clone + 'static,
//   In: Clone + 'static,
// {
//   Observable::<Out> {
//     executor: rxfn(move |s, subscribed| {
//       let _f_next = f.clone();
//       let mut _s_next = s.clone();
//       let mut _s_error = s.clone();
//       let mut _s_complete = s.clone();
//       let inner_subscribed = Rc::new(RefCell::new(true));
//       let _inner_subscribed = Rc::clone(&inner_subscribed);
//       src.inner_subscribe(
//         rxfn(move |x| {
//           if *(subscribed.borrow()) {
//             _s_next.next((&mut *_f_next.borrow_mut())(x));
//           } else {
//             *(inner_subscribed.borrow_mut()) = false;
//             _s_next.discard();
//           }
//         }),
//         rxfn(move |e| _s_error.error(e)),
//         rxfn(move || _s_complete.complete()),
//         _inner_subscribed,
//       );
//     }),
//   }
// }
