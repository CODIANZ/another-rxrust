use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct GroupBy<'a, Item, Key>
where
  Item: Clone + Send + Sync,
  Key: Clone + Send + Sync + Hash + Eq,
{
  key_f: FunctionWrapper<'a, Item, Key>,
}

impl<'a, Item, Key> GroupBy<'a, Item, Key>
where
  Item: Clone + Send + Sync,
  Key: Clone + Send + Sync + Hash + Eq,
{
  pub fn new<F>(f: F) -> GroupBy<'a, Item, Key>
  where
    F: Fn(Item) -> Key + Send + Sync + 'a,
  {
    GroupBy { key_f: FunctionWrapper::new(f) }
  }
  pub fn execute(
    &self,
    source: Observable<'a, Item>,
  ) -> Observable<'a, Observable<'a, Item>> {
    let f = self.key_f.clone();

    Observable::create(move |s| {
      let f = f.clone();

      let sbjmap = Arc::new(RwLock::new(HashMap::<
        Key,
        subjects::Subject<Item>,
      >::new()));

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let sbjmap_next = Arc::clone(&sbjmap);
      let sbjmap_error = Arc::clone(&sbjmap);
      let sbjmap_complete = Arc::clone(&sbjmap);

      source.inner_subscribe(sctl.new_observer(
        move |_, x: Item| {
          let key = f.call(x.clone()); // Umm, can I use it as a reference?
          let sbj = {
            let mut sbjmap = sbjmap_next.write().unwrap();
            if let Some(sbj) = sbjmap.get(&key) {
              sbj.clone()
            } else {
              let sbj = subjects::Subject::<Item>::new();
              sbjmap.insert(key, sbj.clone());
              sctl_next.sink_next(sbj.observable());
              sbj
            }
          };
          sbj.next(x);
        },
        move |_, e| {
          sbjmap_error.read().unwrap().iter().for_each(|x| {
            x.1.error(e.clone());
          });
          sctl_error.sink_error(e);
        },
        move |serial| {
          sbjmap_complete.read().unwrap().iter().for_each(|x| {
            x.1.complete();
          });
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
  pub fn group_by<Key, F>(&self, f: F) -> Observable<'a, Observable<'a, Item>>
  where
    F: Fn(Item) -> Key + Send + Sync + 'a,
    Key: Clone + Send + Sync + Hash + Eq + 'a,
  {
    GroupBy::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::sync::{Arc, RwLock};

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    observables::from_iter(0..10).group_by(|x| x % 3).subscribe(
      move |x| {
        let nn = *n.read().unwrap();
        *n.write().unwrap() += 1;
        x.subscribe(
          move |y| println!("next ({}) - {}", nn, y),
          move |e| println!("error ({}) - {:?}", nn, e),
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
      .group_by(|x| x % 3)
      .subscribe(
        move |x| {
          let nn = *n.read().unwrap();
          *n.write().unwrap() += 1;
          x.subscribe(
            move |y| println!("next ({}) - {}", nn, y),
            move |e| println!("error ({}) - {:?}", nn, e),
            move || println!("complete ({})", nn),
          );
        },
        print_error!(),
        print_complete!(),
      );
  }
}
