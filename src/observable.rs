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

  pub(crate) fn inner_subscribe(
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

  pub fn map<Out>(&self, f: RxFn<dyn FnMut(Item) -> Out>) -> Observable<Out>
  where
    Out: Clone + 'static,
  {
    operators::map(self.clone(), f)
  }

  pub fn flat_map<Out>(&self, f: RxFn<dyn FnMut(Item) -> Observable<Out>>) -> Observable<Out>
  where
    Out: Clone + 'static,
  {
    operators::flat_map(self.clone(), f)
  }
}

mod test {
  use crate::all::*;
  use anyhow::anyhow;
  use std::rc::Rc;

  #[test]
  fn basic() {
    Observable::<i32>::create(rxfn(|mut s, _| {
      for n in 1..10 {
        s.next(n);
      }
      s.complete();
    }))
    .subscribe(
      rxfn(|x| {
        println!("next {}", x);
      }),
      rxfn(|e| {
        println!("error {:}", e);
      }),
      rxfn(|| {
        println!("complete");
      }),
    );
  }

  #[test]
  fn emit_error() {
    Observable::<i32>::create(rxfn(|mut s, _| {
      for n in 1..10 {
        s.next(n);
      }
      s.error(Rc::new(anyhow!("error!!")));
    }))
    .subscribe(
      rxfn(|x| {
        println!("next {}", x);
      }),
      rxfn(|e| {
        println!("error {:?}", e);
      }),
      rxfn(|| {
        println!("complete");
      }),
    );
  }
}
