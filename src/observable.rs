use scheduler::IScheduler;

use crate::{internals::function_wrapper::FunctionWrapper, prelude::*};

#[derive(Clone)]
pub struct Subscription<'a> {
  fn_unsubscribe: FunctionWrapper<'a, (), ()>,
}

impl<'a> Subscription<'a> {
  pub fn new<Unsub>(unsub: Unsub) -> Subscription<'a>
  where
    Unsub: Fn() + Send + Sync + 'a,
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
pub struct Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  source: FunctionWrapper<'a, Observer<'a, Item>, ()>,
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn create<Source>(source: Source) -> Observable<'a, Item>
  where
    Source: Fn(Observer<'a, Item>) + Send + Sync + 'a,
  {
    Observable {
      source: FunctionWrapper::new(source),
    }
  }

  pub(crate) fn inner_subscribe(&self, observer: Observer<'a, Item>) -> Subscription {
    let unsub_observer = observer.clone();
    self.source.call(observer.clone());
    Subscription::new(move || {
      unsub_observer.unsubscribe();
    })
  }

  pub fn subscribe<Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Subscription
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    self.inner_subscribe(Observer::new(next, error, complete))
  }

  pub fn map<Out, F>(&self, f: F) -> Observable<'a, Out>
  where
    F: Fn(Item) -> Out + Send + Sync + 'a,
    Out: Clone + Send + Sync,
  {
    operators::MapOp::new(f).execute(self.clone())
  }

  pub fn flat_map<Out, F>(&self, f: F) -> Observable<'a, Out>
  where
    F: Fn(Item) -> Observable<'a, Out> + Send + Sync + 'a,
    Out: Clone + Send + Sync,
  {
    operators::FlatMapOp::new(f).execute(self.clone())
  }

  pub fn on_error_resume_next<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(RxError) -> Observable<'a, Item> + Send + Sync + 'a,
  {
    operators::OnErrorResumeNextOp::new(f).execute(self.clone())
  }

  pub fn observe_on<S>(&self, s: S) -> Observable<'a, Item>
  where
    S: IScheduler<'a> + Clone + Send + Sync + 'a,
  {
    operators::ObserveOnOp::new(s).execute(self.clone())
  }

  pub fn take(&self, count: usize) -> Observable<'a, Item> {
    operators::TakeOp::new(count).execute(self.clone())
  }

  pub fn take_last(&self, count: usize) -> Observable<'a, Item> {
    operators::TakeLastOp::new(count).execute(self.clone())
  }

  pub fn skip(&self, count: usize) -> Observable<'a, Item> {
    operators::SkipOp::new(count).execute(self.clone())
  }

  pub fn skip_last(&self, count: usize) -> Observable<'a, Item> {
    operators::SkipLastOp::new(count).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use super::Observable;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
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
    let o = Observable::create(|s| {
      thread::spawn(move || {
        for n in 0..100 {
          if !s.is_subscribed() {
            break;
          }
          s.next(n);
        }
        if s.is_subscribed() {
          s.complete();
        }
      });
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
    let o = Observable::create(|s| {
      thread::spawn(move || {
        for n in 0..100 {
          if !s.is_subscribed() {
            println!("break!");
            break;
          }
          s.next(n);
          thread::sleep(time::Duration::from_millis(100));
        }
        if s.is_subscribed() {
          s.complete();
        }
      });
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
