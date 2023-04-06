use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct SequenceEqual<'a, Item>
where
  Item: Clone + Send + Sync,
{
  zip_op: operators::Zip<'a, Item>,
}

impl<'a, Item> SequenceEqual<'a, Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> SequenceEqual<'a, Item> {
    SequenceEqual { zip_op: operators::Zip::new(observables) }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, bool> {
    let zip_op = self.zip_op.clone();

    Observable::create(move |s| {
      let source = source.clone();

      let sctl = StreamController::new(s);

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      zip_op.execute(source).inner_subscribe(sctl.new_observer(
        move |serial, x: Vec<Item>| {
          let check = x.get(0).unwrap();
          if !x.iter().all(|i| i == check) {
            sctl_next.upstream_abort_observe(&serial);
            sctl_next.sink_next(false);
            sctl_next.sink_complete(&serial);
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          sctl_complete.sink_next(true);
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn sequence_equal(
    &self,
    observables: &[Observable<'a, Item>],
  ) -> Observable<'a, bool> {
    SequenceEqual::new(observables).execute(self.clone())
  }
}

#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    fn ob() -> Observable<'static, i32> {
      observables::from_iter(0..10)
    }
    ob().sequence_equal(&[ob(), ob()]).subscribe(
      |x| {
        println!("next - {}", x);
        assert_eq!(x, true);
      },
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn thread() {
    fn ob() -> Observable<'static, i32> {
      observables::from_iter(0..10)
        .observe_on(schedulers::new_thread_scheduler())
    }
    ob().sequence_equal(&[ob(), ob()]).subscribe(
      |x| {
        println!("next - {}", x);
        assert_eq!(x, true);
      },
      print_error!(),
      print_complete!(),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }

  #[test]
  fn ne() {
    fn ob(start: i32, memo: &'static str) -> Observable<'static, i32> {
      observables::from_iter(start..start + 10)
        .observe_on(schedulers::new_thread_scheduler())
        .tap(
          move |x| println!("tap ({}) {}", memo, x),
          junk_error!(),
          junk_complete!(),
        )
    }
    ob(0, "A")
      .sequence_equal(&[ob(0, "B"), ob(1, "C")])
      .subscribe(
        |x| {
          println!("next - {}", x);
          assert_eq!(x, false);
        },
        print_error!(),
        print_complete!(),
      );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
