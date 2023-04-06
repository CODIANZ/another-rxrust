use crate::prelude::*;

#[derive(Clone)]
pub struct Something<T>
where
  T: Clone + Send + Sync,
{
  value: Result<T, RxError>,
}

impl<T> Something<T>
where
  T: Clone + Send + Sync,
{
  pub fn success(x: T) -> Something<T> {
    Something { value: Ok(x) }
  }

  pub fn error(e: RxError) -> Something<T> {
    Something { value: Err(e) }
  }

  pub fn proceed<'a>(self) -> Observable<'a, T> {
    match self.value {
      Ok(x) => observables::just(x),
      Err(e) => observables::error(e),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use std::sync::{Arc, RwLock};

  #[test]
  fn basic() {
    let n = Arc::new(RwLock::new(0));
    let o = Observable::create(|s| {
      *n.write().unwrap() += 1;
      let x = *n.read().unwrap();
      s.next(x);
      s.error(RxError::from_error(x));
    });

    o.map(|x| utils::Something::success(x))
      .on_error_resume_next(|e| {
        if *e.downcast_ref::<i32>().unwrap() > 5 {
          observables::just(utils::Something::error(e)) // pass `retry()` and emit an error
        } else {
          observables::error(e) // trigger `retry()`
        }
      })
      .retry(0)
      .flat_map(|x| x.proceed())
      .subscribe(
        print_next_fmt!("{}"),
        print_error_as!(i32),
        print_complete!(),
      );
  }
}
