use std::{cell::RefCell, rc::Rc};

use crate::all::*;

#[derive(Clone)]
pub struct Observable<Item>
where
  Item: Clone + 'static,
{
  pub(crate) executor: RxFn<dyn Fn(Subscriber<Item>, Rc<RefCell<bool>>)>,
}

impl<Item> Observable<Item>
where
  Item: Clone + 'static,
{
  pub fn create(f: RxFn<dyn Fn(Subscriber<Item>, Rc<RefCell<bool>>)>) -> Observable<Item> {
    Observable { executor: f }
  }

  pub fn subscribe(
    &self,
    next: RxFn<dyn FnMut(Item)>,
    error: RxFn<dyn FnMut(RxError)>,
    complete: RxFn<dyn FnMut()>,
  ) -> Subscription {
    let subscribed = Rc::new(RefCell::new(true));
    self.inner_subscribe(next, error, complete, subscribed)
  }

  fn inner_subscribe(
    &self,
    next: RxFn<dyn FnMut(Item)>,
    error: RxFn<dyn FnMut(RxError)>,
    complete: RxFn<dyn FnMut()>,
    subscribed: Rc<RefCell<bool>>,
  ) -> Subscription {
    let mut subscriber = Subscriber {
      observer: Some(Observer {
        next,
        error,
        complete,
      }),
    };
    if let Ok(executor) = self.executor.try_borrow() {
      let subscriber = subscriber.clone();
      let subscribed = subscribed.clone();
      executor(subscriber, subscribed);
    }
    Subscription {
      fn_unsubscribe: rxfn(move || {
        *(subscribed.borrow_mut()) = false;
        subscriber.discard()
      }),
    }
  }

  pub fn map<Out>(self, f: RxFn<dyn FnMut(Item) -> Out>) -> Observable<Out>
  where
    Out: Clone + 'static,
  {
    let _f = f.clone();
    let mut _self = self.clone();
    Observable::<Out> {
      executor: rxfn(move |s, subscribed| {
        let _f_next = _f.clone();
        let mut _s_next = s.clone();
        let mut _s_error = s.clone();
        let mut _s_complete = s.clone();
        let inner_subscribed = Rc::new(RefCell::new(true));
        let _inner_subscribed = Rc::clone(&inner_subscribed);
        _self.inner_subscribe(
          rxfn(move |x| {
            if *(subscribed.borrow()) {
              _s_next.next((&mut *_f_next.borrow_mut())(x));
            } else {
              *(inner_subscribed.borrow_mut()) = false;
              _s_next.discard();
            }
          }),
          rxfn(move |e| _s_error.error(e)),
          rxfn(move || _s_complete.complete()),
          _inner_subscribed,
        );
      }),
    }
  }

  pub fn flat_map<Out>(self, f: RxFn<dyn FnMut(Item) -> Observable<Out>>) -> Observable<Out>
  where
    Out: Clone + 'static,
  {
    let _f = f.clone();
    let mut _self = self.clone();

    Observable::<Out> {
      executor: rxfn(move |s, subscribed| {
        let counter = Rc::new(RefCell::new(0));
        let source_completed = Rc::new(RefCell::new(false));

        let _counter_is_complete = Rc::clone(&counter);
        let _source_completed_is_complete = Rc::clone(&source_completed);
        let is_complete = Rc::new(RefCell::new(move || {
          *(_counter_is_complete.borrow()) == 0 && *(_source_completed_is_complete.borrow())
        }));

        let _counter_next = Rc::clone(&counter);
        let _is_complete_next = Rc::clone(&is_complete);

        let _source_completed_complete = Rc::clone(&source_completed);
        let _is_complete_complete = Rc::clone(&is_complete);

        let _f_next = _f.clone();
        let mut _s_next = s.clone();
        let mut _s_error = s.clone();
        let mut _s_complete = s.clone();

        let inner_subscribed = Rc::new(RefCell::new(true));
        let _inner_subscribed = Rc::clone(&inner_subscribed);

        _self.inner_subscribe(
          rxfn(move |x| {
            *(_counter_next.borrow_mut()) += 1;
            let _counter_next_complete = Rc::clone(&_counter_next);
            let _is_complete_next_complete = Rc::clone(&_is_complete_next);
            let mut _s_next_next = _s_next.clone();
            let mut _s_next_error = _s_next.clone();
            let mut _s_next_complete = _s_next.clone();
            let _subscribed_next = Rc::clone(&subscribed);

            let _inner_subscribed = Rc::clone(&_inner_subscribed);
            let _inner_innter_subscribed = Rc::clone(&_inner_subscribed);

            ((&mut *_f_next.borrow_mut())(x)).inner_subscribe(
              rxfn(move |x| {
                if *(_subscribed_next.borrow()) {
                  _s_next_next.next(x);
                } else {
                  *(_inner_subscribed.borrow_mut()) = false;
                  _s_next_next.discard();
                }
              }),
              rxfn(move |e| _s_next_error.error(e)),
              rxfn(move || {
                *(_counter_next_complete.borrow_mut()) -= 1;
                let is_complete = &*(_is_complete_next_complete.borrow());
                if is_complete() {
                  _s_next_complete.complete();
                }
              }),
              _inner_innter_subscribed,
            );
          }),
          rxfn(move |e| _s_error.error(e)),
          rxfn(move || {
            *(_source_completed_complete.borrow_mut()) = true;
            let is_complete = &*(_is_complete_complete.borrow());
            if is_complete() {
              _s_complete.complete()
            }
          }),
          inner_subscribed,
        );
      }),
    }
  }
}
