use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct WindowWithCount<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  _item: PhantomData<Item>,
}

impl<'a, Item> WindowWithCount<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> WindowWithCount<Item> {
    WindowWithCount {
      count,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Observable<'a, Item>> {
    let count = self.count;

    Observable::create(move |s| {
      let n = Arc::new(RwLock::new(0));
      let sbj = Arc::new(RwLock::new(subjects::Subject::<Item>::new()));

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let sbj_next = Arc::clone(&sbj);
      let sbj_error = Arc::clone(&sbj);
      let sbj_complete = Arc::clone(&sbj);

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          let mut n = n.write().unwrap();
          if *n == 0 {
            sctl_next.sink_next(sbj_next.read().unwrap().observable());
            sbj_next.read().unwrap().next(x);
            *n += 1;
          } else {
            sbj_next.read().unwrap().next(x);
            *n += 1;
            if *n == count {
              sbj_next.read().unwrap().complete();
              *sbj_next.write().unwrap() = subjects::Subject::<Item>::new();
              *n = 0;
            }
          }
        },
        move |_, e| {
          sbj_error.read().unwrap().error(e.clone());
          sctl_error.sink_error(e);
        },
        move |serial| {
          sbj_complete.read().unwrap().complete();
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
  pub fn window_with_count(&self, count: usize) -> Observable<'a, Observable<'a, Item>> {
    WindowWithCount::new(count).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::sync::{Arc, RwLock};

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    observables::from_iter(0..10)
      .window_with_count(3)
      .subscribe(
        move |x| {
          let nn = *n.read().unwrap();
          *n.write().unwrap() += 1;
          x.subscribe(
            move |y| println!("next ({}) - {}", nn, y),
            move |e| println!("error ({}) - {:?}", nn, e.any_ref()),
            move || println!("complete ({})", nn),
          );
        },
        print_error!(),
        print_complete!(),
      );
  }

  #[test]
  fn error() {
    let n = Arc::new(RwLock::new(0));
    observables::from_iter(0..10)
      .flat_map(|x| {
        if x == 7 {
          observables::error(RxError::from_error("it's 7!!"))
        } else {
          observables::just(x)
        }
      })
      .window_with_count(3)
      .subscribe(
        move |x| {
          let nn = *n.read().unwrap();
          *n.write().unwrap() += 1;
          x.subscribe(
            move |y| println!("next ({}) - {}", nn, y),
            move |e| println!("error ({}) - {:?}", nn, e.cast_ref::<&str>()),
            move || println!("complete ({})", nn),
          );
        },
        print_error!(),
        print_complete!(),
      );
  }
}
