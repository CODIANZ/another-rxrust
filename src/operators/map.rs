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

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(f.call(x));
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
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
