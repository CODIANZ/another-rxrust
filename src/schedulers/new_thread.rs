use super::schedulers::{AsyncScheduler, IScheduler};
use std::thread;

pub struct NewThreadScheduler {
  scheduler: AsyncScheduler,
}

impl NewThreadScheduler {
  pub fn new() -> NewThreadScheduler {
    NewThreadScheduler {
      scheduler: AsyncScheduler::new(),
    }
  }
}

impl IScheduler for NewThreadScheduler {
  fn start(&self) {
    let scheduler = self.scheduler.clone();
    thread::spawn(move || {
      scheduler.scheduling();
    });
  }

  fn stop(&self) {
    self.scheduler.stop();
  }

  fn post<F>(&self, f: F)
  where
    F: Fn() + Clone + Send + Sync + 'static,
  {
    self.scheduler.post(f);
  }
}

#[cfg(test)]
mod test {
  use super::NewThreadScheduler;
  use crate::prelude::schedulers::IScheduler;
  use std::{thread, time};

  #[test]
  fn basic() {
    let s = NewThreadScheduler::new();
    s.start();

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
    println!("stop!!");
    s.stop();
    thread::sleep(time::Duration::from_millis(2000));
  }
}
