use super::schedulers::IScheduler;

#[derive(Clone)]
pub struct DefaultScheduler {}

impl DefaultScheduler {
  pub fn new() -> DefaultScheduler {
    DefaultScheduler {}
  }
}

impl IScheduler<'_> for DefaultScheduler {
  fn post<F>(&self, f: F)
  where
    F: Fn() + Send + Sync,
  {
    f();
  }
}

pub fn default_scheduler() -> DefaultScheduler {
  DefaultScheduler::new()
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use scheduler::IScheduler;
  use std::{thread, time};

  #[test]
  fn basic() {
    {
      let s = schedulers::default_scheduler();

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
    }
  }
}
