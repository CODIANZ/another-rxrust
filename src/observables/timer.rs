use crate::prelude::*;
use scheduler::IScheduler;
use std::{thread, time::Duration};

pub fn timer<'a, Scheduler>(dur: Duration, scheduler: Scheduler) -> Observable<'a, ()>
where
  Scheduler: IScheduler<'a> + Send + Sync + 'a,
{
  Observable::create(move |s| {
    scheduler.post(move || {
      thread::sleep(dur);
      s.next(());
      s.complete();
    })
  })
}

#[cfg(test)]
mod test {
  use super::timer;
  use crate::prelude::{default_scheduler::default_scheduler, schedulers::new_thread_scheduler};
  use std::{thread, time};

  #[test]
  fn basic() {
    timer(time::Duration::from_millis(1000), default_scheduler()).subscribe(
      |_| println!("next"),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    timer(time::Duration::from_millis(1000), new_thread_scheduler()).subscribe(
      |_| println!("next"),
      |e| println!("{:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1500));
  }
}