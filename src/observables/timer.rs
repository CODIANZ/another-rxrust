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

#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::timer(
      time::Duration::from_millis(1000),
      schedulers::default_scheduler(),
    )
    .subscribe(
      print_next!(),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn thread() {
    observables::timer(
      time::Duration::from_millis(1000),
      schedulers::new_thread_scheduler(),
    )
    .subscribe(
      print_next!(),
      print_error!(),
      print_complete!(),
    );
    thread::sleep(time::Duration::from_millis(1500));
  }
}
