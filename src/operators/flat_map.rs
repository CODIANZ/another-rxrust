use std::{
  marker::PhantomData,
  sync::{Arc, RwLock},
};

use crate::all::*;

struct WrapF<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  func: Box<dyn Fn(In) -> Observable<Out> + Send + Sync + 'static>,
}
impl<In, Out> WrapF<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  fn new<F>(func: F) -> WrapF<In, Out>
  where
    F: Fn(In) -> Observable<Out> + Send + Sync + 'static,
    In: Clone + Send + Sync + 'static,
    Out: Clone + Send + Sync + 'static,
  {
    WrapF {
      func: Box::new(func),
    }
  }
  fn call(&self, indata: In) -> Observable<Out> {
    (self.func)(indata)
  }
}

pub struct FlatMapOp<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  wrap_f: Arc<WrapF<In, Out>>,
  _in: PhantomData<In>,
}

impl<In, Out> FlatMapOp<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  pub fn new<F>(f: F) -> FlatMapOp<In, Out>
  where
    F: Fn(In) -> Observable<Out> + Send + Sync + 'static,
  {
    FlatMapOp {
      wrap_f: Arc::new(WrapF::new(f)),
      _in: PhantomData,
    }
  }
  pub fn execute(&self, soruce: Observable<In>) -> Observable<Out> {
    let _f = Arc::clone(&self.wrap_f);
    let _source = Arc::new(soruce);

    Observable::<Out>::create(move |s| {
      struct Work<Out> {
        counter_: Arc<RwLock<i32>>,
        source_completed_: Arc<RwLock<bool>>,
        subscribed_: Arc<RwLock<bool>>,
        subscriber_: Arc<Observer<Out>>,
        subscriptions_: Arc<RwLock<Vec<Subscription>>>,
      }
      impl<Out> Work<Out> {
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
        fn subscriber_next(&self, x: Out) {
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

      let _f_next = Arc::clone(&_f);

      work.subscriptions_add(_source.subscribe(
        move |x| {
          if !work_next.subscribed_get_bool() {
            work_next.subscriber_close();
            return;
          }

          work_next.counter_increment();
          let work_next_next = Arc::clone(&work_next);
          let work_next_error = Arc::clone(&work_next);
          let work_next_complete = Arc::clone(&work_next);

          work_next.subscriptions_add(_f_next.call(x).subscribe(
            move |xx| {
              if work_next_next.subscribed_get_bool() {
                work_next_next.subscriber_next(xx);
              } else {
                work_next_next.subscriber_close();
              }
            },
            move |ee| {
              work_next_error.subscriber_error(ee);
              work_next_error.finish();
            },
            move || {
              work_next_complete.counter_decriment();
              if work_next_complete.is_all_complete() {
                work_next_complete.finish();
              }
            },
          ));
        },
        move |e| {
          work_error.subscriber_error(e);
          work_error.finish();
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
mod test {
  use crate::all::*;
  use std::{
    sync::{Arc, RwLock},
    thread, time,
  };

  #[test]
  fn basic() {
    let o = Observable::<i32>::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
      Subscription::new(|| {})
    });

    o.flat_map(|x| observables::just(x * 2)).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::<i32>::create(|s| {
      let is_subscribed = Arc::new(RwLock::new(true));
      {
        let is_subscribed = Arc::clone(&is_subscribed);
        let s = Arc::new(s);
        thread::spawn(move || {
          for n in 0..100 {
            if !*is_subscribed.read().unwrap() {
              println!("break!");
              break;
            }
            s.next(n);
            thread::sleep(time::Duration::from_millis(100));
          }
          if *is_subscribed.read().unwrap() {
            s.complete();
          }
        });
      }
      Subscription::new(move || {
        *is_subscribed.write().unwrap() = false;
      })
    });

    let sbsc = o
      .flat_map(|x| observables::just(format!("str {}", x)))
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", e),
        || println!("complete"),
      );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
  }

  #[test]
  fn composite() {
    fn o() -> Observable<i32> {
      Observable::<i32>::create(|s| {
        let is_subscribed = Arc::new(RwLock::new(true));
        {
          let is_subscribed = Arc::clone(&is_subscribed);
          let s = Arc::new(s);
          thread::spawn(move || {
            for n in 0..100 {
              if !*is_subscribed.read().unwrap() {
                println!("break!");
                break;
              }
              s.next(n);
              thread::sleep(time::Duration::from_millis(100));
            }
            if *is_subscribed.read().unwrap() {
              s.complete();
            }
          });
        }
        Subscription::new(move || {
          *is_subscribed.write().unwrap() = false;
        })
      })
    }

    o().flat_map(move |_x| o()).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
