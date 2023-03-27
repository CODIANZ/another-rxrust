use crate::{
  internals::{function_wrapper::FunctionWrapper, stream_controller::StreamController},
  prelude::*,
};

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

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let serial = sctl.upstream_prepare_serial();

      sctl.upstream_subscribe(
        &serial,
        source.subscribe(
          move |x| {
            sctl_next.sink_next(f.call(x));
          },
          move |e| {
            sctl_error.sink_error(e);
          },
          move || sctl_complete.sink_complete(&serial),
        ),
      );
      Subscription::new(move || {
        sctl.finalize();
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
