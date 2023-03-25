use std::sync::Arc;

use crate::{
  internals::stream_controller::StreamController,
  observable::{Observable, Subscription},
  prelude::RxError,
};

struct WrapF<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  func: Box<dyn Fn(RxError) -> Observable<Item> + Send + Sync + 'static>,
}
impl<Item> WrapF<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  fn new<F>(func: F) -> WrapF<Item>
  where
    F: Fn(RxError) -> Observable<Item> + Send + Sync + 'static,
  {
    WrapF {
      func: Box::new(func),
    }
  }
  fn call(&self, indata: RxError) -> Observable<Item> {
    (self.func)(indata)
  }
}

pub struct OnErrorResumeNextOp<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  wrap_f: Arc<WrapF<Item>>,
}

impl<Item> OnErrorResumeNextOp<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn new<F>(f: F) -> OnErrorResumeNextOp<Item>
  where
    F: Fn(RxError) -> Observable<Item> + Send + Sync + 'static,
  {
    OnErrorResumeNextOp {
      wrap_f: Arc::new(WrapF::new(f)),
    }
  }
  pub fn execute(&self, soruce: Observable<Item>) -> Observable<Item> {
    let _f = Arc::clone(&self.wrap_f);
    let _source = Arc::new(soruce);

    Observable::<Item>::create(move |s| {
      let sctl = Arc::new(StreamController::new(s));
      let sctl_next = Arc::clone(&sctl);
      let sctl_error = Arc::clone(&sctl);
      let sctl_complete = Arc::clone(&sctl);

      let serial = sctl.upstream_prepare_sereal();

      let _f_error = Arc::clone(&_f);

      sctl.upstream_subscribe(
        &serial,
        _source.subscribe(
          move |x| {
            sctl_next.sink_next(x);
          },
          move |e| {
            let sctl_error_next = Arc::clone(&sctl_error);
            let sctl_error_error = Arc::clone(&sctl_error);
            let sctl_error_complete = Arc::clone(&sctl_error);

            let serial_error = sctl_error.upstream_prepare_sereal();

            sctl_error.upstream_subscribe(
              &serial_error,
              _f_error.call(e).subscribe(
                move |xx| {
                  sctl_error_next.sink_next(xx);
                },
                move |ee| {
                  sctl_error_error.sink_error(ee);
                },
                move || {
                  sctl_error_complete.sink_complete(&serial_error);
                },
              ),
            );
          },
          move || {
            sctl_complete.sink_complete(&serial);
          },
        ),
      );
      let sctl = Arc::clone(&sctl);
      Subscription::new(move || {
        sctl.finalize();
      })
    })
  }
}

#[cfg(test)]
mod tset {
  use crate::prelude::*;
  use anyhow::anyhow;
  use std::sync::Arc;

  #[test]
  fn basic() {
    let o = Observable::<i32>::create(|s| {
      s.next(1);
      s.error(Arc::new(anyhow!("err")));
      Subscription::new(|| {})
    });

    o.on_error_resume_next(|_err| observables::just(100))
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", e),
        || println!("complete"),
      );
  }
}
