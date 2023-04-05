use crate::prelude::*;
use scheduler::IScheduler;
use std::{thread, time::Duration};

pub fn interval<'a, Scheduler, SchedulerCreator>(
  dur: Duration,
  scheduler_ctor: SchedulerCreator,
) -> Observable<'a, u64>
where
  Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
  SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
{
  Observable::create(move |s| {
    let scheduler = scheduler_ctor();
    let scheduler_in_post = scheduler.clone();
    scheduler.post(move || {
      let mut n = 0;
      loop {
        thread::sleep(dur);
        if !s.is_subscribed() {
          break;
        }
        s.next(n);
        n += 1;
      }
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
    observables::interval(
      time::Duration::from_millis(100),
      schedulers::default_scheduler(),
    )
    .take(5)
    .subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    println!("marker");
  }

  #[test]
  fn thread() {
    observables::interval(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    )
    .take(5)
    .subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    println!("marker");
    thread::sleep(time::Duration::from_millis(1500));
  }
}
