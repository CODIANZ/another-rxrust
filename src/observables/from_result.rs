use crate::prelude::*;

pub fn from_result<'a, Item, E>(x: Result<Item, E>) -> Observable<'a, Item>
where
  Item: std::fmt::Debug + Clone + Send + Sync,
  E: std::fmt::Debug + Send + Sync + 'static,
{
  if let Ok(x) = &x {
    observables::just(x.clone())
  } else {
    observables::error(RxError::from_result(x))
  }
}

#[cfg(test)]
mod test {
  use anyhow::anyhow;

  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::from_result(Ok::<_, ()>("abc")).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error_str() {
    observables::from_result(Err::<(), _>("abc")).subscribe(
      print_next!(),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error_io() {
    observables::from_result(Err::<(), _>(std::io::Error::from(
      std::io::ErrorKind::NotFound,
    )))
    .subscribe(
      print_next!(),
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn error_anyhow() {
    observables::from_result(Err::<(), _>(anyhow!("anyhow error"))).subscribe(
      print_next!(),
      print_error!(),
      print_complete!(),
    );
  }
}
