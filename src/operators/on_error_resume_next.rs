use crate::{
  internals::{function_wrapper::FunctionWrapper, stream_controller::StreamController},
  observable::{Observable, Subscription},
  prelude::RxError,
};

pub struct OnErrorResumeNextOp<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  wrap_f: FunctionWrapper<RxError, Observable<Item>>,
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
      wrap_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<Item>) -> Observable<Item> {
    let f = self.wrap_f.clone();

    Observable::<Item>::create(move |s| {
      let f = f.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let serial = sctl.upstream_prepare_serial();
      let serial_upstream = serial.clone();

      sctl.upstream_subscribe(
        &serial,
        source.subscribe(
          move |x| {
            sctl_next.sink_next(x);
          },
          move |e| {
            sctl_error.upstream_unsubscribe(&serial_upstream);

            let sctl_error_next = sctl_error.clone();
            let sctl_error_error = sctl_error.clone();
            let sctl_error_complete = sctl_error.clone();

            let serial_error = sctl_error.upstream_prepare_serial();

            sctl_error.upstream_subscribe(
              &serial_error,
              f.call(e).subscribe(
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

  #[test]
  fn basic() {
    observables::error(RxError::new(anyhow!("err")))
      .on_error_resume_next(|_e| observables::just(1))
      .subscribe(
        move |x| {
          println!("next {}", x);
        },
        |e| {
          println!("error {:}", e.error);
        },
        || {
          println!("complete");
        },
      );
  }

  #[test]
  fn just() {
    observables::just(1)
      .on_error_resume_next(|_e| observables::just(1))
      .subscribe(
        move |x| {
          println!("next {}", x);
        },
        |e| {
          println!("error {:}", e.error);
        },
        || {
          println!("complete");
        },
      );
  }
}
