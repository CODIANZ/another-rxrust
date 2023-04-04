use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;

#[derive(Clone)]
pub struct Filter<'a, Item>
where
  Item: Clone + Send + Sync,
{
  predicate_f: FunctionWrapper<'a, Item, bool>,
}

impl<'a, Item> Filter<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> Filter<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    Filter { predicate_f: FunctionWrapper::new(f) }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let f = self.predicate_f.clone();

    Observable::<Item>::create(move |s| {
      let f = f.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          if f.call(x.clone()) {
            sctl_next.sink_next(x);
          }
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn filter<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    Filter::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.filter(|x| 3 <= x && x <= 5)
      .tap(
        |x| {
          assert!(3 <= x && x <= 5);
        },
        junk_error!(),
        junk_complete!(),
      )
      .subscribe(
        print_next_fmt!("{}"),
        print_error!(),
        print_complete!(),
      );
  }
}
