use crate::prelude::*;

pub struct Using<'a> {
  subscription: Subscription<'a>,
}

impl<'a> Using<'a> {
  pub fn new(subscription: Subscription<'a>) -> Using<'a> {
    Using { subscription }
  }
}

impl<'a> Drop for Using<'a> {
  fn drop(&mut self) {
    self.subscription.unsubscribe();
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    {
      let _using = utils::Using::new(
        observables::interval(
          time::Duration::from_millis(10),
          schedulers::new_thread_scheduler(),
        )
        .subscribe(print_next_fmt!("{}"), print_error!(), print_complete!()),
      );
      thread::sleep(time::Duration::from_millis(500));
    }
    println!("scope out");
    thread::sleep(time::Duration::from_millis(500));
  }
}
