use crate::{
  internals::stream_controller::StreamController,
  prelude::{Observable, Observer, RxError},
};

pub struct TapOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  tap_observer: Observer<'a, Item>,
}

impl<'a, Item> TapOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new<Next, Error, Complete>(next: Next, error: Error, complete: Complete) -> TapOp<'a, Item>
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    TapOp {
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

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[cfg(feature = "anyhow")]
  use anyhow::anyhow;

  #[cfg(feature = "anyhow")]
  fn generate_error() -> RxError {
    RxError::new(anyhow!("anyhow error"))
  }

  #[cfg(not(feature = "anyhow"))]
  fn generate_error() -> RxError {
    RxError::new("string error".to_owned())
  }

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
      |e| println!("tap error {:}", e.error),
      || println!("tap complete"),
    )
    .map(|x| x + 100)
    .subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
  }

  #[test]
  fn error() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
      }
      s.error(generate_error());
    });

    o.tap(
      |x| println!("tap next {}", x),
      |e| println!("tap error {:}", e.error),
      || println!("tap complete"),
    )
    .map(|x| x + 100)
    .subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
  }
}
