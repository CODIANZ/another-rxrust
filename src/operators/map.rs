use crate::{internals::function_wrapper::FunctionWrapper, prelude::*};
use std::{marker::PhantomData, sync::Arc};

pub struct MapOp<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  wrap_f: FunctionWrapper<In, Out>,
  _in: PhantomData<In>,
}

impl<In, Out> MapOp<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  pub fn new<F>(f: F) -> MapOp<In, Out>
  where
    F: Fn(In) -> Out + Send + Sync + 'static,
  {
    MapOp {
      wrap_f: FunctionWrapper::new(f),
      _in: PhantomData,
    }
  }
  pub fn execute(&self, soruce: Observable<In>) -> Observable<Out> {
    let _f = self.wrap_f.clone();
    let _source = Arc::new(soruce);

    Observable::<Out>::create(move |s| {
      let s_next = Arc::clone(&s);
      let s_error = Arc::clone(&s);
      let s_complete = Arc::clone(&s);
      let _f_next = _f.clone();
      let sbsc = _source.subscribe(
        move |x| {
          s_next.next(_f_next.call(x));
        },
        move |e| {
          s_error.error(e);
        },
        move || s_complete.complete(),
      );
      Subscription::new(move || {
        sbsc.unsubscribe();
      })
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
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

    o.map(|x| x * 2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
  }

  #[test]
  fn map_thread() {
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

    let sbsc = o.map(|x| format!("str {}", x)).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }
}
