#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn rxcpp_case5() {
    fn periodical() -> Observable<'static, u64> {
      observables::interval(
        time::Duration::from_micros(100),
        schedulers::new_thread_scheduler(),
      )
      .tap(
        print_next_fmt!("tap {}"),
        junk_error!(),
        junk_complete!(),
      )
    }

    let sbsc = periodical()
      .flat_map(move |_| {
        return periodical().flat_map(move |_| {
          return periodical();
        });
      })
      .take(1)
      .subscribe(
        print_next_fmt!("{}"),
        junk_error!(),
        junk_complete!(),
      );

    while sbsc.is_subscribed() {}
    thread::sleep(time::Duration::from_millis(1000));
  }
}
