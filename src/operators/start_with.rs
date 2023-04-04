use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct StartWith<'a, Item, Iter>
where
  Item: Clone + Send + Sync,
  Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
{
  it: Iter,
  _item: PhantomData<Item>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Item, Iter> StartWith<'a, Item, Iter>
where
  Item: Clone + Send + Sync,
  Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
{
  pub fn new(it: Iter) -> StartWith<'a, Item, Iter> {
    StartWith {
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn start_with<Iter>(&self, iter: Iter) -> Observable<'a, Item>
  where
    Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
  {
    StartWith::new(iter).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn single() {
    let o = Observable::create(|s| {
      for n in 0..5 {
        s.next(n);
      }
      s.complete();
    });

    o.start_with([1000].into_iter()).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
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
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
