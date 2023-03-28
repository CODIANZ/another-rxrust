use super::schedulers::{AsyncFunctionQueue, IScheduler};
use std::thread;

#[derive(Clone)]
pub struct NewThreadScheduler<'a> {
  scheduler: AsyncFunctionQueue<'a>,
}

impl NewThreadScheduler<'_> {
  pub fn new() -> NewThreadScheduler<'static> {
    let scheduler = AsyncFunctionQueue::new();

    let scheduler_thread = scheduler.clone();
    thread::spawn(move || {
      scheduler_thread.scheduling();
    });

    NewThreadScheduler { scheduler }
  }
}

impl IScheduler<'static> for NewThreadScheduler<'static> {
  fn post<F>(&self, f: F)
  where
    F: Fn() + Send + Sync + 'static,
  {
    self.scheduler.post(f);
  }
  fn abort(&self) {
    self.scheduler.stop();
  }
}

pub fn new_thread_scheduler() -> NewThreadScheduler<'static> {
  NewThreadScheduler::new()
}

#[cfg(test)]
mod test {
  use super::NewThreadScheduler;
  use crate::prelude::schedulers::IScheduler;
  use std::{thread, time};

  #[test]
  fn basic() {
    {
      let s = NewThreadScheduler::new();

      s.post(|| {
        println!("#1 start");
        thread::sleep(time::Duration::from_millis(500));
        println!("#1 end");
      });

      s.post(|| {
        println!("#2 start");
        thread::sleep(time::Duration::from_millis(500));
        println!("#2 end");
      });

      s.post(|| {
        println!("#3 start");
        thread::sleep(time::Duration::from_millis(500));
        println!("#3 end");
      });

      thread::sleep(time::Duration::from_millis(700));
    }
    thread::sleep(time::Duration::from_millis(2000));
  }
}
