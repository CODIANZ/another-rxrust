use crate::prelude::*;

pub fn error<'a, Item>(err: RxError) -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::<Item>::create(move |s| {
    s.error(err.clone());
  })
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::error::<String>(RxError::from_error("ERR!")).subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );
  }
}
