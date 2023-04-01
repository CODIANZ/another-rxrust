use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct BufferWithCountOp<Item>
where
  Item: Clone + Send + Sync,
{
  count: usize,
  _item: PhantomData<Item>,
}

impl<'a, Item> BufferWithCountOp<Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(count: usize) -> BufferWithCountOp<Item> {
    assert!(count > 0);
    BufferWithCountOp {
      count,
      _item: PhantomData,
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Vec<Item>> {
    let count = self.count;
    Observable::<Vec<Item>>::create(move |s| {
      let results = Arc::new(RwLock::new(Vec::new()));
      let sctl = StreamController::new(s);

      let sctl_f = sctl.clone();
      let results_f = Arc::clone(&results);
      let register = move |item: Item| {
        let vec = {
          let mut vec = results_f.write().unwrap();
          vec.push(item);
          if vec.len() == count {
            let v = vec.clone();
            vec.clear();
            Some(v)
          } else {
            None
          }
        };
        if let Some(vec) = vec {
          sctl_f.sink_next(vec);
        }
      };

      {
        let register = register.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        source.inner_subscribe(sctl.new_observer(
          move |_, x| register(x),
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |serial| {
            let vec = results.read().unwrap();
            if vec.len() > 0 {
              sctl_complete.sink_next(vec.clone());
            }
            sctl_complete.sink_complete(&serial);
          },
        ));
      }
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn buffer_with_count(&self, count: usize) -> Observable<'a, Vec<Item>> {
    BufferWithCountOp::new(count).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::from_iter(0..10)
      .buffer_with_count(3)
      .subscribe(print_next_fmt!("{:?}"), print_error!(), print_complete!());
    thread::sleep(time::Duration::from_millis(1000));
  }
}
