use crate::internals::function_wrapper::FunctionWrapper;
use std::{
  collections::VecDeque,
  sync::{Arc, Condvar, Mutex, RwLock},
};

struct SchedulerControl<'a> {
  queue: Mutex<VecDeque<FunctionWrapper<'a, (), ()>>>,
  cond: Condvar,
  abort: RwLock<bool>,
}

#[derive(Clone)]
pub struct AsyncScheduler<'a> {
  ctrl: Arc<SchedulerControl<'a>>,
}
impl<'a> AsyncScheduler<'a> {
  pub fn new() -> AsyncScheduler<'a> {
    AsyncScheduler {
      ctrl: Arc::new(SchedulerControl {
        queue: Mutex::new(VecDeque::new()),
        cond: Condvar::new(),
        abort: RwLock::new(false),
      }),
    }
  }

  pub fn scheduling(&self) {
    loop {
      let f = {
        let que_mtx = self.ctrl.queue.lock().unwrap();
        let mut que_mtx = self
          .ctrl
          .cond
          .wait_while(que_mtx, |que| {
            if *self.ctrl.abort.read().unwrap() {
              false
            } else {
              que.is_empty()
            }
          })
          .unwrap();
        if *self.ctrl.abort.read().unwrap() {
          None
        } else {
          que_mtx.pop_front()
        }
      };
      if let Some(f) = f {
        f.call(());
      } else {
        break;
      }
    }
  }

  pub fn post<F>(&self, f: F)
  where
    F: Fn() + Send + Sync + 'a,
  {
    let mut que_mtx = self.ctrl.queue.lock().unwrap();
    que_mtx.push_back(FunctionWrapper::new(move |_| f()));
    self.ctrl.cond.notify_one();
  }

  pub fn stop(&self) {
    let mut que_mtx = self.ctrl.queue.lock().unwrap();
    que_mtx.clear();
    *self.ctrl.abort.write().unwrap() = true;
    self.ctrl.cond.notify_one();
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::schedulers::AsyncScheduler;
  use std::{thread, time};

  #[test]
  fn basic() {
    let scheduler = AsyncScheduler::new();
    let scheduler_thread = scheduler.clone();
    thread::spawn(move || {
      scheduler_thread.scheduling();
    });

    scheduler.post(|| {
      println!("#1 start");
      thread::sleep(time::Duration::from_millis(500));
      println!("#1 end");
    });

    scheduler.post(|| {
      println!("#2 start");
      thread::sleep(time::Duration::from_millis(500));
      println!("#2 end");
    });

    scheduler.post(|| {
      println!("#3 start");
      thread::sleep(time::Duration::from_millis(500));
      println!("#3 end");
    });

    thread::sleep(time::Duration::from_millis(700));
    println!("stop!!");
    scheduler.stop();
    thread::sleep(time::Duration::from_millis(2000));
  }
}
