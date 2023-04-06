use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct Merge<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> Merge<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> Merge<'a, Item> {
    Merge { observables: observables.to_vec() }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let observables = self.observables.clone();
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);

      // prepare subscribers
      let mut sbs = {
        let sctl = sctl.clone();
        Vec::from_iter(
          (0..(observables.len() + 1)).map(move |_| {
            let sctl_next = sctl.clone();
            let sctl_error = sctl.clone();
            let sctl_complete = sctl.clone();

            sctl.new_observer(
              move |_, x| {
                sctl_next.sink_next(x);
              },
              move |_, e| {
                sctl_error.sink_error(e);
              },
              move |serial| sctl_complete.sink_complete(&serial),
            )
          }),
        )
      };

      source.inner_subscribe(sbs.pop().unwrap());
      observables.iter().for_each(|o| {
        o.inner_subscribe(sbs.pop().unwrap());
      });
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn merge(
    &self,
    observables: &[Observable<'a, Item>],
  ) -> Observable<'a, Item> {
    Merge::new(observables).execute(self.clone())
  }
}

#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    fn ob(len: usize, maker: &'static str) -> Observable<String> {
      observables::from_iter(0..len)
        .map(move |x| format!("{} - {} / {}", maker, x + 1, len))
    }

    ob(5, "#1")
      .merge(&[ob(3, "#2"), ob(2, "#3"), ob(6, "#4")])
      .subscribe(
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );
  }

  #[test]
  fn thread() {
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
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );

    thread::sleep(time::Duration::from_millis(1500));
  }
}
