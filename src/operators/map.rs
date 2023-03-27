use crate::{internals::function_wrapper::FunctionWrapper, prelude::*};

pub struct MapOp<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  wrap_f: FunctionWrapper<In, Out>,
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
    }
  }
  pub fn execute(&self, source: Observable<In>) -> Observable<Out> {
    let f = self.wrap_f.clone();

    Observable::<Out>::create(move |s| {
      let f = f.clone();

      let s_next = s.clone();
      let s_error = s.clone();
      let s_complete = s.clone();
      let sbsc = source.subscribe(
        move |x| {
          s_next.next(f.call(x));
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
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
      Subscription::new(|| {})
    });

    o.map(|x| x * 2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
  }

  #[test]
  fn map_thread() {
    let o = Observable::create(|s| {
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
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }
}
