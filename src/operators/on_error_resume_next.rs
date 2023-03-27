use crate::{
  internals::{function_wrapper::FunctionWrapper, stream_controller::StreamController},
  observable::Observable,
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

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(x);
        },
        move |serial, e| {
          sctl_error.upstream_abort_observe(&serial);

          let sctl_error_next = sctl_error.clone();
          let sctl_error_error = sctl_error.clone();
          let sctl_error_complete = sctl_error.clone();

          f.call(e).inner_subscribe(sctl_error.new_observer(
            move |_, xx| {
              sctl_error_next.sink_next(xx);
            },
            move |_, ee| {
              sctl_error_error.sink_error(ee);
            },
            move |serial| {
              sctl_error_complete.sink_complete(&serial);
            },
          ));
        },
        move |serial| {
          sctl_complete.sink_complete(&serial);
        },
      ));
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
