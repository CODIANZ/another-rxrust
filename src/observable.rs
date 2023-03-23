use crate::{all::RxError, observer::Observer};
use std::sync::{Arc, RwLock};

pub struct Subscription {
  fn_unsubscribe: Box<dyn Fn()>,
}

impl Subscription {
  pub fn new<Unsub>(unsub: Unsub) -> Subscription
  where
    Unsub: Fn() + Send + Sync + 'static,
  {
    Subscription {
      fn_unsubscribe: Box::new(unsub),
    }
  }
  pub fn unsubscribe(&self) {
    (self.fn_unsubscribe)();
  }
}

pub struct Emitter<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  func: Box<dyn Fn(Arc<Observer<Item>>, Arc<RwLock<bool>>) + Send + Sync + 'static>,
}

impl<Item> Emitter<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn new<F>(f: F) -> Emitter<Item>
  where
    F: Fn(Arc<Observer<Item>>, Arc<RwLock<bool>>) + Send + Sync + 'static,
  {
    Emitter { func: Box::new(f) }
  }
  pub fn execute(&self, observer: Arc<Observer<Item>>, is_subscribed: Arc<RwLock<bool>>) {
    (self.func)(observer, is_subscribed);
  }
}

#[derive(Clone)]
pub struct Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  source: Arc<Emitter<Item>>,
}

impl<Item> Observable<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn create<Source>(source: Source) -> Observable<Item>
  where
    Source: Fn(Arc<Observer<Item>>, Arc<RwLock<bool>>) + Send + Sync + 'static,
  {
    Observable {
      source: Arc::new(Emitter::new(source)),
    }
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
    self.inner_subscribe(
      Observer::new(next, error, complete),
      Arc::new(RwLock::new(true)),
    )
  }

  pub(crate) fn inner_subscribe(
    &self,
    observer: Observer<Item>,
    is_subscribed: Arc<RwLock<bool>>,
  ) -> Subscription {
    let observer = Arc::new(observer);
    let unsub_observer = Arc::clone(&observer);
    let unsub_is_subscribed = Arc::clone(&is_subscribed);
    self.source.execute(observer, is_subscribed);
    Subscription::new(move || {
      unsub_observer.close();
      if let Ok(mut state) = unsub_is_subscribed.write() {
        *state = true;
      }
    })
  }

  pub fn map<Out, F>(&self, f: F) -> Observable<Out>
  where
    F: Fn(Item) -> Out + Send + Sync + 'static,
    Out: Clone + Send + Sync + 'static,
  {
    struct FWrap<In, Out>
    where
      In: Clone + Send + Sync + 'static,
      Out: Clone + Send + Sync + 'static,
    {
      func: Box<dyn Fn(In) -> Out + Send + Sync + 'static>,
    }
    impl<In, Out> FWrap<In, Out>
    where
      In: Clone + Send + Sync + 'static,
      Out: Clone + Send + Sync + 'static,
    {
      fn new<F>(func: F) -> FWrap<In, Out>
      where
        F: Fn(In) -> Out + Send + Sync + 'static,
        In: Clone + Send + Sync + 'static,
        Out: Clone + Send + Sync + 'static,
      {
        FWrap {
          func: Box::new(func),
        }
      }
      fn call(&self, indata: In) -> Out {
        (self.func)(indata)
      }
    }

    let fwrap = Arc::new(FWrap::new(f));
    let _self = self.clone();
    Observable::<Out>::create(move |s, _| {
      let s_next = Arc::clone(&s);
      let s_error = Arc::clone(&s);
      let s_complete = Arc::clone(&s);
      let s_fwrap = Arc::clone(&fwrap);
      _self.subscribe(
        move |x| {
          s_next.next(s_fwrap.call(x));
        },
        move |e| {
          s_error.error(e);
        },
        move || s_complete.complete(),
      );
    })
  }
}

mod test {
  use super::Observable;
  use std::{sync::Arc, thread, time};

  #[test]
  fn basic() {
    let o = Observable::<i32>::create(|s, _is_subscribed| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::<i32>::create(|s, _is_subscribed| {
      let s = Arc::new(s);
      thread::spawn(move || {
        for n in 0..100 {
          s.next(n);
        }
        s.complete();
      });
    });

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
    println!("started");
  }

  #[test]
  fn unsubscribe() {
    let o = Observable::<i32>::create(|s, _is_subscribed| {
      let s = Arc::new(s);
      thread::spawn(move || {
        for n in 0..100 {
          thread::sleep(time::Duration::from_millis(100));
          s.next(n);
        }
        s.complete();
      });
    });

    let sbsc = o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
    println!("started");
    thread::sleep(time::Duration::from_millis(1000));
    sbsc.unsubscribe();
  }

  #[test]
  fn map() {
    let o = Observable::<i32>::create(|s, _is_subscribed| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.map(|x| x * 2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
  }

  #[test]
  fn map_thread() {
    let o = Observable::<i32>::create(|s, _is_subscribed| {
      let s = Arc::new(s);
      thread::spawn(move || {
        for n in 0..10 {
          thread::sleep(time::Duration::from_millis(100));
          s.next(n);
        }
        s.complete();
      });
    });

    o.map(|x| format!("str {}", x)).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
