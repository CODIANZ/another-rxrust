use crate::{
  internals::{function_wrapper::FunctionWrapper, stream_controller::StreamController},
  prelude::*,
};

pub struct FlatMapOp<In, Out>
where
  In: Clone + Send + Sync + 'static,
  Out: Clone + Send + Sync + 'static,
{
  wrap_f: FunctionWrapper<In, Observable<Out>>,
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
      wrap_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<In>) -> Observable<Out> {
    let _f = self.wrap_f.clone();
    let _source = source.clone();

    Observable::<Out>::create(move |s| {
      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      let serial = sctl.upstream_prepare_serial();

      let _f_next = _f.clone();

      sctl.upstream_subscribe(
        &serial,
        _source.subscribe(
          move |x| {
            let sctl_next_next = sctl_next.clone();
            let sctl_next_error = sctl_next.clone();
            let sctl_next_complete = sctl_next.clone();

            let serial_next = sctl_next.upstream_prepare_serial();

            sctl_next.upstream_subscribe(
              &serial_next,
              _f_next.call(x).subscribe(
                move |xx| {
                  sctl_next_next.sink_next(xx);
                },
                move |ee| {
                  sctl_next_error.sink_error(ee);
                },
                move || {
                  sctl_next_complete.sink_complete(&serial_next);
                },
              ),
            );
          },
          move |e| {
            sctl_error.sink_error(e);
          },
          move || {
            sctl_complete.sink_complete(&serial);
          },
        ),
      );
      Subscription::new(move || {
        sctl.finalize();
      })
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
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
      |e| println!("error {:}", e.error),
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
        |e| println!("error {:}", e.error),
        || println!("complete"),
      );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
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
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(1000));
  }
}
