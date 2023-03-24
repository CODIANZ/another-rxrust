use std::sync::{Arc, RwLock};

use crate::{
  all::RxError,
  observable::{Observable, Subscription},
  observer::Observer,
};

struct WrapF<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  func: Box<dyn Fn(RxError) -> Observable<Item> + Send + Sync + 'static>,
}
impl<Item> WrapF<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  fn new<F>(func: F) -> WrapF<Item>
  where
    F: Fn(RxError) -> Observable<Item> + Send + Sync + 'static,
  {
    WrapF {
      func: Box::new(func),
    }
  }
  fn call(&self, indata: RxError) -> Observable<Item> {
    (self.func)(indata)
  }
}

pub struct OnErrorResumeNextOp<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  wrap_f: Arc<WrapF<Item>>,
}

impl<Item> OnErrorResumeNextOp<Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn new<F>(f: F) -> OnErrorResumeNextOp<Item>
  where
    F: Fn(RxError) -> Observable<Item> + Send + Sync + 'static,
  {
    OnErrorResumeNextOp {
      wrap_f: Arc::new(WrapF::new(f)),
    }
  }
  pub fn execute(&self, soruce: Observable<Item>) -> Observable<Item> {
    let _f = Arc::clone(&self.wrap_f);
    let _source = Arc::new(soruce);

    Observable::<Item>::create(move |s| {
      struct Work<Item> {
        counter_: Arc<RwLock<i32>>,
        source_completed_: Arc<RwLock<bool>>,
        subscribed_: Arc<RwLock<bool>>,
        subscriber_: Arc<Observer<Item>>,
        subscriptions_: Arc<RwLock<Vec<Subscription>>>,
      }
      impl<Item> Work<Item> {
        fn is_all_complete(&self) -> bool {
          let counter = *self.counter_.read().unwrap();
          let source_completed = *self.source_completed_.read().unwrap();
          counter == 0 && source_completed
        }
        fn counter_increment(&self) {
          let mut counter = self.counter_.write().unwrap();
          *counter += 1;
        }
        fn counter_decriment(&self) {
          let mut counter = self.counter_.write().unwrap();
          *counter -= 1;
        }
        fn source_completed_set_true(&self) {
          let mut source_completed = self.source_completed_.write().unwrap();
          *source_completed = true;
        }
        fn subscribed_set_false(&self) {
          let mut subscribed = self.subscribed_.write().unwrap();
          *subscribed = false;
        }
        fn subscribed_get_bool(&self) -> bool {
          *self.subscribed_.read().unwrap()
        }
        fn subscriber_next(&self, x: Item) {
          self.subscriber_.next(x);
        }
        fn subscriber_error(&self, e: RxError) {
          self.subscriber_.error(e);
        }
        fn subscriber_complete(&self) {
          self.subscriber_.complete();
        }
        fn subscriber_close(&self) {
          self.subscriber_.close();
        }
        fn subscriptions_add(&self, sbsc: Subscription) {
          self.subscriptions_.write().unwrap().push(sbsc);
        }
        fn subscriptions_unsubscribe_all(&self) {
          self.subscriptions_.read().unwrap().iter().for_each(|x| {
            x.unsubscribe();
          });
          self.subscriptions_.write().unwrap().clear();
        }
        fn finish(&self) {
          self.subscriber_complete();
          self.subscribed_set_false();
          self.subscriptions_unsubscribe_all();
        }
      }

      let work = Arc::new(Work {
        counter_: Arc::new(RwLock::new(0)),
        source_completed_: Arc::new(RwLock::new(false)),
        subscribed_: Arc::new(RwLock::new(true)),
        subscriber_: Arc::clone(&s),
        subscriptions_: Arc::new(RwLock::new(Vec::new())),
      });

      let work_next = Arc::clone(&work);
      let work_error = Arc::clone(&work);
      let work_complete = Arc::clone(&work);

      let _f_error = Arc::clone(&_f);

      work.subscriptions_add(_source.subscribe(
        move |x| {
          if !work_next.subscribed_get_bool() {
            work_next.subscriber_close();
            return;
          }
          work_next.subscriber_next(x);
        },
        move |e| {
          let work_error_next = Arc::clone(&work_error);
          let work_error_error = Arc::clone(&work_error);
          let work_error_complete = Arc::clone(&work_error);

          work_error.subscriptions_add(_f_error.call(e).subscribe(
            move |xx| {
              if work_error_next.subscribed_get_bool() {
                work_error_next.counter_increment();
                work_error_next.subscriber_next(xx);
              } else {
                work_error_next.subscriber_close();
              }
            },
            move |ee| {
              work_error_error.subscriber_error(ee);
              work_error_error.finish();
            },
            move || {
              work_error_complete.counter_decriment();
              if work_error_complete.is_all_complete() {
                work_error_complete.finish();
              }
            },
          ));
        },
        move || {
          work_complete.source_completed_set_true();
          if work_complete.is_all_complete() {
            work_complete.finish();
          }
        },
      ));
      let work = Arc::clone(&work);
      Subscription::new(move || {
        work.finish();
      })
    })
  }
}

#[cfg(test)]
mod tset {
  use crate::all::*;
  use anyhow::anyhow;
  use std::sync::Arc;

  #[test]
  fn basic() {
    let o = Observable::<i32>::create(|s| {
      s.next(1);
      s.error(Arc::new(anyhow!("err")));
      Subscription::new(|| {})
    });

    o.on_error_resume_next(|_err| observables::just(100))
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", e),
        || println!("complete"),
      );
  }
}
