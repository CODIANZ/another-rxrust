use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct All<'a, Item>
where
  Item: Clone + Send + Sync,
{
  filter_op: operators::Filter<'a, Item>,
}

impl<'a, Item> All<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> All<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    All {
      filter_op: operators::Filter::new(move |x| !f(x)),
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, bool> {
    let filter_op = self.filter_op.clone();

    Observable::create(move |s| {
      let source = source.clone();

      let sctl = StreamController::new(s);

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      filter_op
        .execute(source)
        .take(1)
        .inner_subscribe(sctl.new_observer(
          move |serial, _| {
            sctl_next.upstream_abort_observe(&serial);
            sctl_next.sink_next(false);
            sctl_next.sink_complete(&serial);
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
  Item: Clone + Send + Sync,
{
  pub fn all<F>(&self, f: F) -> Observable<'a, bool>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    All::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::repeat(86).take(10).all(|x| x == 86).subscribe(
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
    observables::repeat(86).take(10).all(|x| x != 86).subscribe(
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
