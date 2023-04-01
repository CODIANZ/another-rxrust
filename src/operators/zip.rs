use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  collections::{HashMap, VecDeque},
  sync::{Arc, RwLock},
};

pub struct ZipOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> ZipOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> ZipOp<'a, Item> {
    ZipOp {
      observables: observables.to_vec(),
    }
  }
  pub fn execute(&self, source: Observable<'a, Item>) -> Observable<'a, Vec<Item>> {
    let observables = self.observables.clone();
    Observable::<Vec<Item>>::create(move |s| {
      let results = Arc::new(RwLock::new({
        let mut r = HashMap::<usize, VecDeque<Item>>::new();
        for n in 0..(observables.len() + 1) {
          r.insert(n, VecDeque::<Item>::new());
        }
        r
      }));

      let s_f = s.clone();
      let results_f = Arc::clone(&results);
      let register = move |id: &usize, item: Item| {
        results_f
          .write()
          .unwrap()
          .get_mut(id)
          .unwrap()
          .push_back(item);

        let re = Arc::clone(&results_f);
        let get = move || {
          let mut re = re.write().unwrap();
          let filled = re.iter().filter(|x| x.1.len() > 0).count();
          if filled == re.len() {
            let mut v = Vec::new();
            for items in re.iter_mut() {
              v.push(items.1.pop_front().unwrap());
            }
            Some(v)
          } else {
            None
          }
        };
        while let Some(items) = get() {
          s_f.next(items);
        }
      };

      let sctl = StreamController::new(s);

      {
        let mut id = 1usize;
        for o in &observables {
          {
            let id = id;
            let register = register.clone();
            let sctl_error = sctl.clone();
            let sctl_complete = sctl.clone();

            o.inner_subscribe(sctl.new_observer(
              move |_, x| register(&id, x),
              move |_, e| {
                sctl_error.sink_error(e);
              },
              move |serial| {
                sctl_complete.sink_complete(&serial);
              },
            ));
          }
          id += 1;
        }
      }
      {
        let id = 0;
        let register = register.clone();
        let sctl_error = sctl.clone();
        let sctl_complete = sctl.clone();

        source.inner_subscribe(sctl.new_observer(
          move |_, x| register(&id, x),
          move |_, e| {
            sctl_error.sink_error(e);
          },
          move |serial| {
            sctl_complete.sink_complete(&serial);
          },
        ));
      }
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::from_iter(0..10)
      .observe_on(schedulers::new_thread_scheduler())
      .zip(&[
        observables::from_iter(10..20).observe_on(schedulers::new_thread_scheduler()),
        observables::from_iter(20..30).observe_on(schedulers::new_thread_scheduler()),
      ])
      .subscribe(print_next_fmt!("{:?}"), print_error!(), print_complete!());
    thread::sleep(time::Duration::from_millis(1000));
  }
}
