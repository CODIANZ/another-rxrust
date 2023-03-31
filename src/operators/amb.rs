use std::sync::{Arc, RwLock};

use crate::internals::stream_controller::*;
use crate::prelude::*;

pub struct AmbOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> AmbOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> AmbOp<'a, Item> {
    AmbOp {
      observables: observables.to_vec(),
    }
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

      for o in &observables {
        let is_win_next = is_win.clone();
        let is_win_error = is_win.clone();
        let is_win_complete = is_win.clone();

        let sctl_next = sctl.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        o.inner_subscribe(sctl.new_observer(
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
        ));
      }

      let is_win_next = is_win.clone();
      let is_win_error = is_win.clone();
      let is_win_complete = is_win.clone();

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
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
      .take(10)
      .take_last(len)
      .map(move |x| format!("{} - {} / {}", maker, x + 1, len))
    }

    ob(5, "#1")
      .amb(&[ob(3, "#2"), ob(2, "#3"), ob(6, "#4")])
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", error_to_string(&e)),
        || println!("complete"),
      );

    thread::sleep(time::Duration::from_millis(1500));
  }
}