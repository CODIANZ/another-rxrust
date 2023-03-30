use crate::internals::stream_controller::*;
use crate::prelude::*;

pub struct MergeOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> MergeOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> MergeOp<'a, Item> {
    MergeOp {
      observables: observables.to_vec(),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let observables = self.observables.clone();
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);

      for o in &observables {
        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        o.inner_subscribe(sctl.new_observer(
          move |_, x| {
            sctl_next.sink_next(x);
          },
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |serial| sctl_complete.sink_complete(&serial),
        ));
      }

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(x);
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
  use crate::tests::common::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    fn ob(len: usize, maker: &'static str) -> Observable<String> {
      observables::interval(
        time::Duration::from_millis(100),
        schedulers::new_thread_scheduler(),
      )
      .take(len)
      .map(move |x| format!("{} - {} / {}", maker, x + 1, len))
    }

    ob(5, "#1")
      .merge(&[ob(3, "#2"), ob(2, "#3"), ob(6, "#4")])
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", error_to_string(&e)),
        || println!("complete"),
      );

    thread::sleep(time::Duration::from_millis(1500));
  }
}
