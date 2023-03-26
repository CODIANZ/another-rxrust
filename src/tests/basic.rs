#[cfg(test)]
mod test {
  use std::{thread, time};

  use crate::prelude::*;
  use anyhow::anyhow;

  #[test]
  fn basic() {
    fn ob() -> Observable<i32> {
      Observable::create(|s| {
        s.next(1);
        s.next(2);
        s.next(3);
        s.complete();
        Subscription::new(|| {})
      })
    }

    ob()
      .observe_on(schedulers::new_thread())
      .flat_map(|x| match x {
        1 => observables::empty(),
        2 => observables::just(x),
        3 => observables::error(RxError::new(anyhow!("err"))),
        _ => observables::just(x),
      })
      .map(|x| format!("{}", x))
      .on_error_resume_next(|e| ob().map(move |x| format!("resume {:} {}", e.error, x)))
      .subscribe(
        |x| {
          println!("next {}", x);
        },
        |e| {
          println!("error {:}", e.error);
        },
        || {
          println!("complete");
        },
      );

    // TODO: not be completed!
    thread::sleep(time::Duration::from_millis(500));
  }
}
