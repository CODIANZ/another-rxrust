use crate::prelude::*;

pub fn range<'a>(initial: i64, count: i64) -> Observable<'a, i64> {
  Observable::create(move |s| {
    for n in initial..(initial + count) {
      if !s.is_subscribed() {
        break;
      }
      s.next(n.clone());
    }
    s.complete();
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::range(5, 5).subscribe(print_next_fmt!("{}"), print_error!(), print_complete!());
  }
}
