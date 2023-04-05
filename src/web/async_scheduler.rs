use crate::prelude::{schedulers::IScheduler, web::*};
use std::time;

#[derive(Clone)]
pub struct AsyncScheduler {}

impl AsyncScheduler {
  pub fn new() -> AsyncScheduler {
    AsyncScheduler {}
  }
}

impl IScheduler<'static> for AsyncScheduler {
  fn post<F>(&self, f: F)
  where
    F: Fn() + Send + Sync + 'static,
  {
    set_timeout(f, time::Duration::from_millis(0));
  }
  fn abort(&self) {}
}

pub fn async_scheduler<'a>() -> fn() -> AsyncScheduler {
  || AsyncScheduler::new()
}
