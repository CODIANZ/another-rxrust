use crate::internals::function_wrapper::FunctionWrapper;
use std::{
  collections::VecDeque,
  sync::{Arc, Condvar, Mutex, RwLock},
};

struct AsyncFunctionQueueData<'a> {
  queue: Mutex<VecDeque<FunctionWrapper<'a, (), ()>>>,
  cond: Condvar,
  abort: RwLock<bool>,
}

#[derive(Clone)]
pub struct AsyncFunctionQueue<'a> {
  data: Arc<AsyncFunctionQueueData<'a>>,
}
impl<'a> AsyncFunctionQueue<'a> {
  pub fn new() -> AsyncFunctionQueue<'a> {
    AsyncFunctionQueue {
      data: Arc::new(AsyncFunctionQueueData {
        queue: Mutex::new(VecDeque::new()),
        cond: Condvar::new(),
        abort: RwLock::new(false),
      }),
    }
  }

  pub fn scheduling(&self) {
    loop {
      let f = {
        let que_mtx = self.data.queue.lock().unwrap();
        let mut que_mtx = self
          .data
          .cond
          .wait_while(que_mtx, |que| {
            if *self.data.abort.read().unwrap() {
              false
            } else {
              que.is_empty()
            }
          })
          .unwrap();
        if *self.data.abort.read().unwrap() {
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
    let mut que_mtx = self.data.queue.lock().unwrap();
    que_mtx.push_back(FunctionWrapper::new(move |_| f()));
    self.data.cond.notify_one();
  }

  pub fn stop(&self) {
    let mut que_mtx = self.data.queue.lock().unwrap();
    que_mtx.clear();
    *self.data.abort.write().unwrap() = true;
    self.data.cond.notify_one();
  }
}

#[cfg(test)]
mod test {
  use super::AsyncFunctionQueue;
  use std::{thread, time};

  #[test]
  fn basic() {
    let scheduler = AsyncFunctionQueue::new();
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
