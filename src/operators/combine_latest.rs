use crate::internals::function_wrapper::FunctionWrapper;
use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct CombineLatest<'a, Item, Out>
where
  Item: Clone + Send + Sync,
  Out: Clone + Send + Sync,
{
  zip_op: operators::Zip<'a, Item>,
  combine_f: FunctionWrapper<'a, Vec<Item>, Out>,
}

impl<'a, Item, Out> CombineLatest<'a, Item, Out>
where
  Item: Clone + Send + Sync,
  Out: Clone + Send + Sync,
{
  pub fn new<F>(observables: &[Observable<'a, Item>], f: F) -> CombineLatest<'a, Item, Out>
  where
    F: Fn(Vec<Item>) -> Out + Send + Sync + 'a,
  {
    CombineLatest {
      zip_op: operators::Zip::new(observables),
      combine_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Out> {
    let zip_op = self.zip_op.clone();
    let combine_f = self.combine_f.clone();

    Observable::create(move |s| {
      let source = source.clone();
      let combine_f = combine_f.clone();

      let sctl = StreamController::new(s);

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      zip_op.execute(source).inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(combine_f.call(x));
        },
        move |_, e| {
          sctl_error.sink_error(e);
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
  pub fn combine_latest<Out, F>(
    &self,
    observables: &[Observable<'a, Item>],
    f: F,
  ) -> Observable<'a, Out>
  where
    Out: Clone + Send + Sync,
    F: Fn(Vec<Item>) -> Out + Send + Sync + 'a,
  {
    CombineLatest::new(observables, f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::from_iter(0..10)
      .observe_on(schedulers::new_thread_scheduler())
      .combine_latest(
        &[
          observables::from_iter(10..20).observe_on(schedulers::new_thread_scheduler()),
          observables::from_iter(20..30).observe_on(schedulers::new_thread_scheduler()),
        ],
        |v| format!("function {:?}", v),
      )
      .subscribe(print_next_fmt!("{:?}"), print_error!(), print_complete!());
    thread::sleep(time::Duration::from_millis(1000));
  }
}
