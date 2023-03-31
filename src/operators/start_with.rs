use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

pub struct StartWithOp<'a, Item, Iter>
where
  Item: Clone + Send + Sync,
  Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
{
  it: Iter,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Item, Iter> StartWithOp<'a, Item, Iter>
where
  Item: Clone + Send + Sync,
  Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
{
  pub fn new(it: Iter) -> StartWithOp<'a, Item, Iter> {
    StartWithOp {
      it,
      _item: PhantomData,
      _lifetime: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Item> {
    let it = self.it.clone();
    Observable::<Item>::create(move |s| {
      for n in it.clone() {
        if !s.is_subscribed() {
          break;
        }
        s.next(n);
      }

      if s.is_subscribed() {
        let sctl = StreamController::new(s);
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
      }
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;

  #[test]
  fn single() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
      }
      s.complete();
    });

    o.start_with([1000].into_iter()).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn multiple() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
      }
      s.complete();
    });

    o.start_with(-5..0).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }
}
