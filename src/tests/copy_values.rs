#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::DestructChecker;
  use std::{thread, time};

  #[test]
  fn basic() {
    observables::just(DestructChecker::new("A")).subscribe(
      junk_next!(),
      junk_error!(),
      junk_complete!(),
    );
  }

  #[test]
  fn use_take() {
    observables::just(DestructChecker::new("A"))
      .take(1)
      .subscribe(
        junk_next!(),
        junk_error!(),
        junk_complete!(),
      );
  }

  #[test]
  fn use_observe_on() {
    {
      observables::just(DestructChecker::new("A"))
        .observe_on(schedulers::new_thread_scheduler())
        .subscribe(
          junk_next!(),
          junk_error!(),
          junk_complete!(),
        );
    }
    println!("out of scope and wait");
    thread::sleep(time::Duration::from_millis(300));
    println!("done");
  }

  #[test]
  fn use_subject() {
    let sbj = subjects::Subject::<DestructChecker>::new();

    sbj.observable().subscribe(
      print_next_fmt!("{}"),
      junk_error!(),
      junk_complete!(),
    );

    sbj.observable().subscribe(
      print_next_fmt!("{}"),
      junk_error!(),
      junk_complete!(),
    );

    sbj.next(DestructChecker::new("A"));
    sbj.complete();
  }
}
