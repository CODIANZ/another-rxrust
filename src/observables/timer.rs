use crate::prelude::*;
use scheduler::IScheduler;
use std::{thread, time::Duration};

pub fn timer<'a, Scheduler, SchedulerCreator>(
  dur: Duration,
  scheduler_ctor: SchedulerCreator,
) -> Observable<'a, ()>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
  SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
{
  Observable::create(move |s| {
    let scheduler = scheduler_ctor();
    let scheduler_in_post = scheduler.clone();
    scheduler.post(move || {
      thread::sleep(dur);
      s.next(());
      s.complete();
      scheduler_in_post.abort();
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
