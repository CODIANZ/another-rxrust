use crate::prelude::*;
use subject::Subject;

pub struct PublishOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  sbj: subjects::Subject<'a, Item>,
  source: Observable<'a, Item>,
}

impl<'a, Item> PublishOp<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(source: Observable<'a, Item>) -> PublishOp<'a, Item> {
    PublishOp {
      sbj: Subject::<Item>::new(),
      source,
    }
  }

  pub fn observable(&self) -> Observable<'a, Item> {
    self.sbj.observable()
  }

  pub fn ref_count(&self) -> usize {
    self.sbj.ref_count()
  }

  pub fn connect(&self) -> Subscription {
    let sbj_next = self.sbj.clone();
    let sbj_error = self.sbj.clone();
    let sbj_complete = self.sbj.clone();

    self.source.subscribe(
      move |x| {
        sbj_next.next(x);
      },
      move |e| {
        sbj_error.error(e);
      },
      move || {
        sbj_complete.complete();
      },
    )
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
    let o = observables::interval(time::Duration::from_millis(100), new_thread_scheduler())
      .tap(print_next_fmt!("tap {}"), print_error!(), print_complete!())
      .publish();
    let obs = o.observable();

    println!("start #1");
    let sbsc1 = obs.subscribe(print_next_fmt!("#1 {}"), print_error!(), print_complete!());
    println!("ref_count {}", o.ref_count());
    assert_eq!(o.ref_count(), 1);

    println!("connect");
    let breaker = o.connect();
    thread::sleep(time::Duration::from_millis(500));

    println!("start #1");
    let sbsc2 = obs.subscribe(print_next_fmt!("#2 {}"), print_error!(), print_complete!());
    println!("ref_count {}", o.ref_count());
    assert_eq!(o.ref_count(), 2);

    thread::sleep(time::Duration::from_millis(500));

    println!("end #1");
    sbsc1.unsubscribe();
    println!("ref_count {}", o.ref_count());
    assert_eq!(o.ref_count(), 1);
    thread::sleep(time::Duration::from_millis(500));

    sbsc2.unsubscribe();
    println!("ref_count {}", o.ref_count());
    assert_eq!(o.ref_count(), 0);
    thread::sleep(time::Duration::from_millis(500));

    println!("braker");
    breaker.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }
}
