use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Amb<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> Amb<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> Amb<'a, Item> {
    Amb { observables: observables.to_vec() }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let observables = self.observables.clone();
    Observable::<Item>::create(move |s| {
      let sctl = StreamController::new(s);
      let winner = Arc::new(RwLock::new(None::<i32>));

      let winner_check = winner.clone();
      let is_win = move |serial: &i32| -> bool {
        let mut w = winner_check.write().unwrap();
        if let Some(w) = &*w {
          *serial == *w
        } else {
          *w = Some(serial.clone());
          true
        }
      };

      // prepare subscribers
      let mut sbs = {
        let sctl = sctl.clone();
        Vec::from_iter(
          (0..(observables.len() + 1)).map(move |_| {
            let is_win_next = is_win.clone();
            let is_win_error = is_win.clone();
            let is_win_complete = is_win.clone();

            let sctl_next = sctl.clone();
            let sctl_error = sctl.clone();
            let sctl_complete = sctl.clone();

            sctl.new_observer(
              move |serial, x| {
                if is_win_next(&serial) {
                  sctl_next.sink_next(x);
                } else {
                  sctl_next.upstream_abort_observe(&serial);
                }
              },
              move |serial, e| {
                if is_win_error(&serial) {
                  sctl_error.sink_error(e);
                } else {
                  sctl_error.upstream_abort_observe(&serial);
                }
              },
              move |serial| {
                if is_win_complete(&serial) {
                  sctl_complete.sink_complete(&serial);
                } else {
                  sctl_complete.upstream_abort_observe(&serial);
                }
              },
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
  pub fn amb(
    &self,
    observables: &[Observable<'a, Item>],
  ) -> Observable<'a, Item> {
    Amb::new(observables).execute(self.clone())
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
      .amb(&[ob(3, "#2"), ob(2, "#3"), ob(6, "#4")])
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
      .amb(&[ob(3, "#2"), ob(2, "#3"), ob(6, "#4")])
      .subscribe(
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );

    thread::sleep(time::Duration::from_millis(1500));
  }
}
