use crate::prelude::*;
use subject::Subject;

#[derive(Clone)]
pub struct Publish<'a, Item>
where
  Item: Clone + Send + Sync,
{
  sbj: subjects::Subject<'a, Item>,
  source: Observable<'a, Item>,
}

impl<'a, Item> Publish<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new(source: Observable<'a, Item>) -> Publish<'a, Item> {
    Publish { sbj: Subject::<Item>::new(), source }
  }

  pub fn observable(&self) -> Observable<'a, Item> {
    self.sbj.observable()
  }

  pub fn connect(&self) -> Subscription<'a> {
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn publish(&self) -> publish::Publish<'a, Item> {
    Publish::new(self.clone())
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
    .publish();
    let obs = o.observable();

    println!("start #1");
    let sbsc1 = obs.subscribe(
      print_next_fmt!("#1 {}"),
      print_error!(),
      print_complete!(),
    );

    println!("connect");
    let breaker = o.connect();
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

    println!("braker");
    breaker.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }

  #[test]
  fn first_connect() {
    let o = observables::interval(
      time::Duration::from_millis(100),
      new_thread_scheduler(),
    )
    .tap(
      print_next_fmt!("tap {}"),
      print_error!(),
      print_complete!(),
    )
    .publish();
    let obs = o.observable();

    println!("connect");
    let breaker = o.connect();
    thread::sleep(time::Duration::from_millis(500));

    println!("start #1");
    let sbsc1 = obs.subscribe(
      print_next_fmt!("#1 {}"),
      print_error!(),
      print_complete!(),
    );

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

    println!("braker");
    breaker.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }
}
