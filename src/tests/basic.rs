#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;
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

    observables::from_iter(1..10)
      .observe_on(schedulers::new_thread_scheduler())
      .flat_map(|x| match x {
        1 => observables::empty(),
        2 => observables::just(x),
        3 => ob().map(move |y| (y + x)),
        4 => observables::error(generate_error()),
        _ => observables::never(),
      })
      .map(|x| format!("{}", x))
      .on_error_resume_next(|e| ob().map(move |x| format!("resume {:} {}", error_to_string(&e), x)))
      .subscribe(
        |x| {
          println!("next {}", x);
        },
        |e| {
          println!("error {}", error_to_string(&e));
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
        println!("error {:}", error_to_string(&e));
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
        println!("error {:}", error_to_string(&e));
      },
      move || {
        println!("complete {}", xxx.data);
      },
    );
  }
}
