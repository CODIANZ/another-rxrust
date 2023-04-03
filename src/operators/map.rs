use crate::internals::{function_wrapper::*, stream_controller::*};
use crate::prelude::*;

#[derive(Clone)]
pub struct Map<'a, In, Out>
where
  In: Clone + Send + Sync,
  Out: Clone + Send + Sync,
{
  map_f: FunctionWrapper<'a, In, Out>,
}

impl<'a, In, Out> Map<'a, In, Out>
where
  In: Clone + Send + Sync,
  Out: Clone + Send + Sync,
{
  pub fn new<F>(f: F) -> Map<'a, In, Out>
  where
    F: Fn(In) -> Out + Send + Sync + 'a,
  {
    Map {
      map_f: FunctionWrapper::new(f),
    }
  }
  pub fn execute(&self, source: Observable<'a, In>) -> Observable<'a, Out> {
    let f = self.map_f.clone();

    Observable::<Out>::create(move |s| {
      let f = f.clone();

      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(f.call(x));
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn map<Out, F>(&self, f: F) -> Observable<'a, Out>
  where
    F: Fn(Item) -> Out + Send + Sync + 'a,
    Out: Clone + Send + Sync,
  {
    Map::new(f).execute(self.clone())
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

    o.map(|x| x * 2).subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn map_thread() {
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
    let binding = o.map(|x| format!("str {}", x));
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
