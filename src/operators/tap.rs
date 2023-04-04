use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct Tap<'a, Item>
where
  Item: Clone + Send + Sync,
{
  tap_observer: Observer<'a, Item>,
}

impl<'a, Item> Tap<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<Next, Error, Complete>(
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Tap<'a, Item>
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    Tap {
      tap_observer: Observer::new(next, error, complete),
    }
  }

  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let tap_observer = self.tap_observer.clone();
    Observable::create(move |s| {
      let sctl = StreamController::new(s);
      let source_next = source.clone();

      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let tap_observer_next = tap_observer.clone();
      let tap_observer_error = tap_observer.clone();
      let tap_observer_complete = tap_observer.clone();
      source_next.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          tap_observer_next.next(x.clone());
          sctl_next.sink_next(x);
        },
        move |_, e| {
          tap_observer_error.error(e.clone());
          sctl_error.sink_error(e);
        },
        move |serial| {
          tap_observer_complete.complete();
          sctl_complete.sink_complete(&serial)
        },
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn tap<Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Observable<'a, Item>
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    Tap::new(next, error, complete).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
      }
      s.complete();
    });

    o.tap(
      |x| println!("tap next {}", x),
      |e| println!("tap error {:?}", e),
      || println!("tap complete"),
    )
    .map(|x| x + 100)
    .subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
      }
      s.error(RxError::from_error("ERR!"));
    });

    o.tap(
      |x| println!("tap next {}", x),
      |e| println!("tap error {:?}", e),
      || println!("tap complete"),
    )
    .map(|x| x + 100)
    .subscribe(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );
  }
}
