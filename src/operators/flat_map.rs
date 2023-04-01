use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;

#[derive(Clone)]
pub struct FlatMapOp<'a, In, Out>
where
  In: Clone + Send + Sync,
  Out: Clone + Send + Sync,
{
  flatmap_f: FunctionWrapper<'a, In, Observable<'a, Out>>,
}

impl<'a, In, Out> FlatMapOp<'a, In, Out>
where
  In: Clone + Send + Sync + 'a,
  Out: Clone + Send + Sync + 'a,
{
  pub fn new<F>(f: F) -> FlatMapOp<'a, In, Out>
  where
    F: Fn(In) -> Observable<'a, Out> + Send + Sync + 'a,
  {
    FlatMapOp {
      flatmap_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<'a, In>) -> Observable<'a, Out> {
    let f = self.flatmap_f.clone();

    Observable::create(move |s| {
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

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn flat_map<Out, F>(&self, f: F) -> Observable<'a, Out>
  where
    F: Fn(Item) -> Observable<'a, Out> + Send + Sync + 'a,
    Out: Clone + Send + Sync,
  {
    FlatMapOp::new(f).execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;
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
      |e| println!("error {:}", error_to_string(&e)),
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

    let binding = o.flat_map(|x| observables::just(format!("str {}", x)));
    let sbsc = binding.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }

  #[test]
  fn composite() {
    fn o() -> Observable<'static, i32> {
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

    let binding = o().flat_map(move |_x| o());
    let sbsc = binding.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
    thread::sleep(time::Duration::from_millis(500));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(500));
  }
}
