// use std::{cell::RefCell, rc::Rc};

// use crate::all::*;

// pub fn flat_map<In, Out>(
//   src: Observable<In>,
//   f: RxFn<dyn FnMut(In) -> Observable<Out>>,
// ) -> Observable<Out>
// where
//   Out: Clone + 'static,
//   In: Clone + 'static,
// {
//   Observable::<Out> {
//     executor: rxfn(move |s, subscribed| {
//       struct Work<Out> {
//         counter_: i32,
//         source_completed_: bool,
//         subscribed_: Rc<RefCell<bool>>,
//         subscriber_: Rc<RefCell<Subscriber<Out>>>,
//       }

//       impl<Out> Work<Out> {
//         fn is_all_complete(&self) -> bool {
//           self.counter_ == 0 && self.source_completed_
//         }
//         fn counter_increment(&mut self) {
//           self.counter_ += 1;
//         }
//         fn counter_decriment(&mut self) {
//           self.counter_ -= 1;
//         }
//         fn source_completed_set_true(&mut self) {
//           self.source_completed_ = true;
//         }
//         fn subscribed_set_false(&mut self) {
//           *self.subscribed_.borrow_mut() = false;
//         }
//         fn subscribed_get_bool(&self) -> bool {
//           *self.subscribed_.borrow()
//         }
//         fn subscribed(&self) -> Rc<RefCell<bool>> {
//           Rc::clone(&self.subscribed_)
//         }
//         fn subscriber_next(&self, x: Out) {
//           self.subscriber_.borrow().next(x);
//         }
//         fn subscriber_error(&self, e: RxError) {
//           self.subscriber_.borrow_mut().error(e);
//         }
//         fn subscriber_complete(&self) {
//           self.subscriber_.borrow_mut().complete();
//         }
//         fn subscriber_discard(&self) {
//           self.subscriber_.borrow_mut().discard();
//         }
//       }

//       let work = Rc::new(RefCell::new(Work {
//         counter_: 0,
//         source_completed_: false,
//         subscribed_: Rc::clone(&subscribed),
//         subscriber_: Rc::new(RefCell::new(s)),
//       }));

//       let w_next = Rc::clone(&work);
//       let w_error = Rc::clone(&work);
//       let w_complete = Rc::clone(&work);

//       let f_next = f.clone();
//       let subscribed = work.borrow().subscribed();

//       src.inner_subscribe(
//         rxfn(move |x| {
//           let w_inner_next = Rc::clone(&w_next);
//           let w_inner_error = Rc::clone(&w_next);
//           let w_innter_complete = Rc::clone(&w_next);
//           let subscribed = w_next.borrow().subscribed();

//           w_next.borrow_mut().counter_increment();
//           ((&mut *f_next.borrow_mut())(x)).inner_subscribe(
//             rxfn(move |x| {
//               if w_inner_next.borrow().subscribed_get_bool() {
//                 w_inner_next.borrow().subscriber_next(x);
//               } else {
//                 w_inner_next.borrow_mut().subscribed_set_false();
//                 w_inner_next.borrow_mut().subscriber_discard();
//               }
//             }),
//             rxfn(move |e| w_inner_error.borrow_mut().subscriber_error(e)),
//             rxfn(move || {
//               w_innter_complete.borrow_mut().counter_decriment();
//               if w_innter_complete.borrow().is_all_complete() {
//                 w_innter_complete.borrow_mut().subscriber_complete();
//               }
//             }),
//             subscribed,
//           );
//         }),
//         rxfn(move |e| w_error.borrow_mut().subscriber_error(e)),
//         rxfn(move || {
//           w_complete.borrow_mut().source_completed_set_true();
//           if w_complete.borrow().is_all_complete() {
//             w_complete.borrow_mut().subscriber_complete();
//           }
//         }),
//         subscribed,
//       );
//     }),
//   }
// }

// mod test {
//   use crate::all::*;
//   use anyhow::anyhow;
//   use std::rc::Rc;

//   #[test]
//   fn basic() {
//     let o = Observable::<i32>::create(rxfn(|mut s, _| {
//       for n in 1..10 {
//         s.next(n);
//       }
//       s.complete();
//     }));

//     let oo = o.clone();

//     let _sbsc = oo
//       .flat_map(rxfn(move |x| match x {
//         1 => observables::just(2).map(rxfn(|x| format!("str {}", x))),
//         2 => Observable::create(rxfn(move |mut s, _| {
//           s.next(x * 10);
//           s.next(x * 11);
//           s.complete();
//         }))
//         .map(rxfn(|x| format!("str {}", x))),
//         3 => o
//           .clone()
//           .map(rxfn(|x| 1000 + x))
//           .map(rxfn(|x| format!("str {}", x))),
//         4 => observables::just(x)
//           .map(rxfn(|x| x * 10))
//           .map(rxfn(|x| format!("str {}", x))),
//         5 => observables::empty(),
//         9 => observables::error(Rc::new(anyhow!("it's nine!!"))),
//         _ => observables::never(),
//       }))
//       .subscribe(
//         rxfn(|x| {
//           println!("next {}", x);
//         }),
//         rxfn(|e| {
//           println!("error {:}", e);
//         }),
//         rxfn(|| {
//           println!("complete");
//         }),
//       );
//   }
// }
