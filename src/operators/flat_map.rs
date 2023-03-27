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
    let f = self.wrap_f.clone();

    Observable::<Out>::create(move |s| {
      let f = f.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          let sctl_next_next = sctl_next.clone();
          let sctl_next_error = sctl_next.clone();
          let sctl_next_complete = sctl_next.clone();

          f.call(x).inner_subscribe(sctl_next.new_observer(
            move |_, xx| {
              sctl_next_next.sink_next(xx);
            },
            move |_, ee| {
              sctl_next_error.sink_error(ee);
            },
            move |serial| {
              sctl_next_complete.sink_complete(&serial);
            },
          ));
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| {
          sctl_complete.sink_complete(&serial);
        },
      ));
    })
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.flat_map(|x| observables::just(x * 2)).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::create(|s| {
      thread::spawn(move || {
        for n in 0..100 {
          if !s.is_subscribed() {
            println!("break!");
            break;
          }
          s.next(n);
          thread::sleep(time::Duration::from_millis(100));
        }
        if s.is_subscribed() {
          s.complete();
        }
      });
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
      Observable::create(|s| {
        thread::spawn(move || {
          for n in 0..100 {
            if !s.is_subscribed() {
              println!("break!");
              break;
            }
            s.next(n);
            thread::sleep(time::Duration::from_millis(100));
          }
          if s.is_subscribed() {
            s.complete();
          }
        });
      })
    }

    let sbsc = o().flat_map(move |_x| o()).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", e.error),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }
}
