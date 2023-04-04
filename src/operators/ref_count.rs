use crate::prelude::*;
use std::sync::{Arc, RwLock};
use subject::Subject;

#[derive(Clone)]
pub struct RefCount<'a, Item>
where
  Item: Clone + Send + Sync,
{
  subject: subjects::Subject<'a, Item>,
  source: Observable<'a, Item>,
  subscription: Arc<RwLock<Option<Subscription<'a>>>>,
}

impl<'a, Item> RefCount<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(source: Observable<'a, Item>) -> RefCount<'a, Item> {
    let _self = RefCount {
      subject: Subject::<Item>::new(),
      source,
      subscription: Arc::new(RwLock::new(None)),
    };
    _self.set_ref_count();
    _self
  }

  pub fn observable(&self) -> Observable<'a, Item> {
    self.subject.observable()
  }

  fn set_ref_count(&self) {
    {
      let subscription = Arc::clone(&self.subscription);
      self.subject.set_on_unsubscribe(move |count| {
        if count == 0 {
          if let Some(sbsc) = &*subscription.read().unwrap() {
            sbsc.unsubscribe();
          }
        }
      });
    }

    let source = self.source.clone();
    let subject = self.subject.clone();
    let subscription = Arc::clone(&self.subscription);

    self.subject.set_on_subscribe(move |count| {
      if count == 1 {
        // connect
        let sbj_next = subject.clone();
        let sbj_error = subject.clone();
        let sbj_complete = subject.clone();

        let mut subscription = subscription.write().unwrap();
        if subscription.is_some() {
          panic!("publish.ref_count(): already connected!");
        }

        *subscription = Some(source.subscribe(
          move |x| {
            sbj_next.next(x);
          },
          move |e| {
            sbj_error.error(e);
          },
          move || {
            sbj_complete.complete();
          },
        ));
      }
    });
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn ref_count(&self) -> ref_count::RefCount<'a, Item> {
    RefCount::new(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::{print_complete, print_error, print_next_fmt};
  use schedulers::new_thread_scheduler;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = observables::interval(
      time::Duration::from_millis(100),
      new_thread_scheduler(),
    )
    .tap(
      print_next_fmt!("tap {}"),
      print_error!(),
      print_complete!(),
    )
    .ref_count();
    let obs = o.observable();

    println!("start #1");
    let sbsc1 = obs.subscribe(
      print_next_fmt!("#1 {}"),
      print_error!(),
      print_complete!(),
    );

    thread::sleep(time::Duration::from_millis(500));

    println!("start #1");
    let sbsc2 = obs.subscribe(
      print_next_fmt!("#2 {}"),
      print_error!(),
      print_complete!(),
    );

    thread::sleep(time::Duration::from_millis(500));

    println!("end #1");
    sbsc1.unsubscribe();

    thread::sleep(time::Duration::from_millis(500));

    println!("end #2");
    sbsc2.unsubscribe();

    thread::sleep(time::Duration::from_millis(500));
  }
}
