use crate::internals::function_wrapper::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  source: FunctionWrapper<'a, Observer<'a, Item>, ()>,
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn create<Source>(source: Source) -> Observable<'a, Item>
  where
    Source: Fn(Observer<'a, Item>) + Send + Sync + 'a,
  {
    Observable { source: FunctionWrapper::new(source) }
  }

  pub(crate) fn inner_subscribe(
    &self,
    observer: Observer<'a, Item>,
  ) -> Subscription<'a> {
    let unsub_observer = observer.clone();
    let issub_observer = observer.clone();
    self.source.call(observer.clone());
    Subscription::new(
      move || {
        unsub_observer.unsubscribe();
      },
      move || issub_observer.is_subscribed(),
    )
  }

  pub fn subscribe<Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Subscription<'a>
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    self.inner_subscribe(Observer::new(next, error, complete))
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

    o.subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );

    o.subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::create(|s| {
      thread::spawn(move || {
        for n in 0..100 {
          if !s.is_subscribed() {
            break;
          }
          s.next(n);
        }
        if s.is_subscribed() {
          s.complete();
        }
      });
    });

    o.subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    println!("started");
  }

  #[test]
  fn unsubscribe() {
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

    let sbsc = o.subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
    println!("started");
    thread::sleep(time::Duration::from_millis(1000));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(1000));
  }

  #[test]
  fn move_to_closure() {
    let o = Observable::create(|s| {
      s.next(1);
      s.complete();
    });
    let oo = o.clone(); // prepare for `move`ing to closure
    o.flat_map(move |_| {
      // Be sure to clone and use the moved `oo`.
      let ooo = oo.clone(); // prepare for `move`ing to closure

      // `oo` must be cloned or an error will occur.
      oo.clone().flat_map(move |_| {
        // Be sure to clone and use the moved `ooo`.
        return ooo.clone();
      })
    })
    .subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
