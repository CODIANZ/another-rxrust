use crate::internals::stream_controller::*;
use crate::prelude::*;
use std::{
  collections::VecDeque,
  sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct Zip<'a, Item>
where
  Item: Clone + Send + Sync,
{
  observables: Vec<Observable<'a, Item>>,
}

impl<'a, Item> Zip<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(observables: &[Observable<'a, Item>]) -> Zip<'a, Item> {
    Zip { observables: observables.to_vec() }
  }
  pub fn execute(
    &self,
    source: Observable<'a, Item>,
  ) -> Observable<'a, Vec<Item>> {
    let observables = self.observables.clone();
    Observable::<Vec<Item>>::create(move |s| {
      let sctl = StreamController::new(s);

      let results = Arc::new(RwLock::new({
        let mut r = Vec::<VecDeque<Item>>::new();
        (0..(observables.len() + 1)).for_each(|_| {
          r.push(VecDeque::<Item>::new());
        });
        r
      }));

      let sctl_f = sctl.clone();
      let results_f = Arc::clone(&results);
      let register = move |id: &usize, item: Item| {
        results_f
          .write()
          .unwrap()
          .get_mut(id.clone())
          .unwrap()
          .push_back(item);

        let re = Arc::clone(&results_f);
        let get = move || {
          let mut re = re.write().unwrap();
          let filled = re.iter().filter(|x| x.len() > 0).count();
          if filled == re.len() {
            let mut v = Vec::new();
            for items in re.iter_mut() {
              v.push(items.pop_front().unwrap());
            }
            Some(v)
          } else {
            None
          }
        };
        while let Some(items) = get() {
          if !sctl_f.is_subscribed() {
            break;
          }
          sctl_f.sink_next(items);
        }
      };

      // prepare subscribers
      let mut sbs = {
        let sctl = sctl.clone();
        VecDeque::from_iter(
          (0..(observables.len() + 1)).map(move |id| {
            let register = register.clone();
            let sctl_error = sctl.clone();
            let sctl_complete = sctl.clone();
            sctl.new_observer(
              move |_, x| register(&id, x),
              move |_, e| {
                sctl_error.sink_error(e);
              },
              move |serial| {
                sctl_complete.sink_complete(&serial);
              },
            )
          }),
        )
      };

      source.inner_subscribe(sbs.pop_front().unwrap());
      observables.iter().for_each(|o| {
        o.inner_subscribe(sbs.pop_front().unwrap());
      });
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn zip(
    &self,
    observables: &[Observable<'a, Item>],
  ) -> Observable<'a, Vec<Item>> {
    Zip::new(observables).execute(self.clone())
  }
}

#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let ob = observables::from_iter(0..10);

    ob.zip(&[ob.map(|x| x + 10), ob.map(|x| x + 20)]).subscribe(
      print_next_fmt!("{:?}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn thread() {
    observables::from_iter(0..10)
      .observe_on(schedulers::new_thread_scheduler())
      .zip(&[
        observables::from_iter(10..20)
          .observe_on(schedulers::new_thread_scheduler()),
        observables::from_iter(20..30)
          .observe_on(schedulers::new_thread_scheduler()),
      ])
      .subscribe(
        print_next_fmt!("{:?}"),
        print_error!(),
        print_complete!(),
      );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
