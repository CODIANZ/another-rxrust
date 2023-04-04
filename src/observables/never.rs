use crate::prelude::*;

pub fn never<'a, Item>() -> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  Observable::<Item>::create(|_| {})
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::never::<String>().subscribe(print_next!(), print_error!(), print_complete!());
  }
}
