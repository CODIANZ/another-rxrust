#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::generate_error;

  #[test]
  fn basic() {
    observables::just(1).subscribe(junk_next!(), junk_error!(), junk_complete!());
  }

  #[test]
  #[should_panic]
  fn panic_error() {
    observables::error::<()>(generate_error()).subscribe(
      junk_next!(),
      panic_error!(),
      junk_complete!(),
    );
  }

  #[test]
  fn print() {
    observables::just(1).subscribe(print_next!(), print_error!(), print_complete!());
  }

  #[test]
  fn print_debug() {
    observables::just(()).subscribe(print_next_fmt!("{:?}"), print_error!(), print_complete!());
  }

  #[test]
  fn print_display() {
    observables::just(1).subscribe(print_next_fmt!("{}"), print_error!(), print_complete!());
  }

  #[test]
  fn print_error() {
    observables::error::<()>(generate_error()).subscribe(
      print_next!(),
      print_error!(),
      print_complete!(),
    );
  }
}