use scheduler::IScheduler;

use crate::{internals::function_wrapper::FunctionWrapper, prelude::*};

#[derive(Clone)]
pub struct Subscription {
  fn_unsubscribe: FunctionWrapper<(), ()>,
}

impl Subscription {
  pub fn new<Unsub>(unsub: Unsub) -> Subscription
  where
    Unsub: Fn() + Send + Sync + 'static,
  {
    Subscription {
      fn_unsubscribe: FunctionWrapper::new(move |_| unsub()),
    }
  }
  pub fn unsubscribe(&self) {
    self.fn_unsubscribe.call_and_clear_if_available(());
  }
}

#[derive(Clone)]
pub struct Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  source: FunctionWrapper<Observer<Item>, Subscription>,
}

impl<Item> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn create<Source>(source: Source) -> Observable<Item>
  where
    Source: Fn(Observer<Item>) -> Subscription + Send + Sync + 'static,
  {
    Observable {
      source: FunctionWrapper::new(source),
    }
  }

  pub(crate) fn inner_subscribe(&self, observer: Observer<Item>) -> Subscription {
    let unsub_observer = observer.clone();
    let subscription = self.source.call(observer.clone());
    Subscription::new(move || {
      unsub_observer.close();
      subscription.unsubscribe();
    })
  }

  pub fn subscribe<Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Subscription
  where
    Next: Fn(Item) + Send + Sync + 'static,
    Error: Fn(RxError) + Send + Sync + 'static,
    Complete: Fn() + Send + Sync + 'static,
  {
    self.inner_subscribe(Observer::new(next, error, complete))
  }

  pub fn map<Out, F>(&self, f: F) -> Observable<Out>
  where
    F: Fn(Item) -> Out + Send + Sync + 'static,
    Out: Clone + Send + Sync + 'static,
  {
    operators::MapOp::new(f).execute(self.clone())
  }

  pub fn flat_map<Out, F>(&self, f: F) -> Observable<Out>
  where
    F: Fn(Item) -> Observable<Out> + Send + Sync + 'static,
    Out: Clone + Send + Sync + 'static,
  {
    operators::FlatMapOp::new(f).execute(self.clone())
  }

  pub fn on_error_resume_next<F>(&self, f: F) -> Observable<Item>
  where
    F: Fn(RxError) -> Observable<Item> + Send + Sync + 'static,
  {
    operators::OnErrorResumeNextOp::new(f).execute(self.clone())
  }

  pub fn observe_on<S>(&self, s: S) -> Observable<Item>
  where
    S: IScheduler + Clone + Send + Sync + 'static,
  {
    operators::ObserveOnOp::new(s).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use super::{Observable, Subscription};
  use std::{
    sync::{Arc, RwLock},
    thread, time,
  };

  #[test]
  fn basic() {
    let o = Observable::<i32>::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
      Subscription::new(|| {})
    });

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::<i32>::create(|s| {
      let is_subscribed = Arc::new(RwLock::new(true));
      {
        let is_subscribed = Arc::clone(&is_subscribed);
        let s = Arc::new(s);
        thread::spawn(move || {
          for n in 0..100 {
            if !*is_subscribed.read().unwrap() {
              break;
            }
            s.next(n);
          }
          if *is_subscribed.read().unwrap() {
            s.complete();
          }
        });
      }
      Subscription::new(move || {
        *is_subscribed.write().unwrap() = false;
      })
    });

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    println!("started");
  }

  #[test]
  fn unsubscribe() {
    let o = Observable::<i32>::create(|s| {
      let is_subscribed = Arc::new(RwLock::new(true));
      {
        let is_subscribed = Arc::clone(&is_subscribed);
        let s = Arc::new(s);
        thread::spawn(move || {
          for n in 0..100 {
            if !*is_subscribed.read().unwrap() {
              println!("break!");
              break;
            }
            s.next(n);
            thread::sleep(time::Duration::from_millis(100));
          }
          if *is_subscribed.read().unwrap() {
            s.complete();
          }
        });
      }
      Subscription::new(move || {
        *is_subscribed.write().unwrap() = false;
      })
    });

    let sbsc = o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    println!("started");
    thread::sleep(time::Duration::from_millis(1000));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(1000));
  }
}
