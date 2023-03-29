#[cfg(test)]
mod test {
  use crate::prelude::*;
  use anyhow::anyhow;
  use std::{thread, time};

  #[test]
  fn basic() {
    fn ob() -> Observable<'static, i32> {
      Observable::create(|s| {
        s.next(100);
        s.next(200);
        s.complete();
      })
    }

    observables::from_iter(vec![1, 2, 3, 4, 5].into_iter())
      .observe_on(schedulers::new_thread_scheduler())
      .flat_map(|x| match x {
        1 => observables::empty(),
        2 => observables::just(x),
        3 => ob().map(move |y| (y + x)),
        4 => observables::error(RxError::new(anyhow!("err"))),
        _ => observables::never(),
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

    thread::sleep(time::Duration::from_millis(500));
  }

  #[test]
  fn outside_var() {
    #[derive(Clone)]
    struct OutsideVar {
      data: String,
    }
    let x = OutsideVar {
      data: "abc".to_owned(),
    };

    let xx = x.clone();
    let xxx = x.clone();

    let jx = observables::just(x);

    jx.subscribe(
      |x| {
        println!("next {}", x.data);
      },
      |e| {
        println!("error {:}", e.error);
      },
      move || {
        println!("complete {}", xx.data);
      },
    );

    jx.subscribe(
      |x| {
        println!("next {}", x.data);
      },
      |e| {
        println!("error {:}", e.error);
      },
      move || {
        println!("complete {}", xxx.data);
      },
    );
  }
}
