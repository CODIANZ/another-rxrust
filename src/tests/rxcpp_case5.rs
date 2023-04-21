#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn rxcpp_case5() {
    let periodical = observables::interval(
      time::Duration::from_micros(100),
      schedulers::new_thread_scheduler(),
    )
    .tap(
      print_next_fmt!("tap {}"),
      junk_error!(),
      junk_complete!(),
    );

    let periodical2 = periodical.clone();
    let sbsc = periodical
      .flat_map(move |_| {
        let periodical3 = periodical2.clone();
        return periodical2.clone().flat_map(move |_| {
          return periodical3.clone();
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
