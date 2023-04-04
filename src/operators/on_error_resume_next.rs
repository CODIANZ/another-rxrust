use crate::internals::{
  function_wrapper::FunctionWrapper, stream_controller::StreamController,
};
use crate::prelude::*;

#[derive(Clone)]
pub struct OnErrorResumeNext<'a, Item>
where
  Item: Clone + Send + Sync,
{
  resume_f: FunctionWrapper<'a, RxError, Observable<'a, Item>>,
}

impl<'a, Item> OnErrorResumeNext<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> OnErrorResumeNext<'a, Item>
  where
    F: Fn(RxError) -> Observable<'a, Item> + Send + Sync + 'a,
  {
    OnErrorResumeNext { resume_f: FunctionWrapper::new(f) }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let f = self.resume_f.clone();

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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn on_error_resume_next<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(RxError) -> Observable<'a, Item> + Send + Sync + 'a,
  {
    OnErrorResumeNext::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod tset {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::error(RxError::from_error("ERR!"))
      .on_error_resume_next(|_e| observables::just(1))
      .subscribe(
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );
  }

  #[test]
  fn just() {
    observables::just(1)
      .on_error_resume_next(|_e| observables::just(1))
      .subscribe(
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );
  }
}
