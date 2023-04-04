use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AsyncSubject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  subject: Arc<subject::Subject<'a, Item>>,
}

impl<'a, Item> AsyncSubject<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn new() -> AsyncSubject<'a, Item> {
    AsyncSubject {
      subject: Arc::new(subjects::Subject::new()),
    }
  }

  pub fn next(&self, item: Item) {
    self.subject.next(item);
  }
  pub fn error(&self, err: RxError) {
    self.subject.error(err);
  }
  pub fn complete(&self) {
    self.subject.complete();
  }
  pub fn observable(&self) -> Observable<'a, Item> {
    self.subject.observable().take_last(1).clone()
  }
}

#[cfg(test)]
mod tset {
  use crate::prelude::*;

  #[test]
  fn basic() {
    let sbj = subjects::AsyncSubject::new();

    sbj.observable().subscribe(
      print_next_fmt!("{}"),
      print_error!(),
      print_complete!(),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.next(3);
    sbj.complete();
  }

  #[test]
  fn error() {
    let sbj = subjects::AsyncSubject::new();

    sbj.observable().subscribe(
      print_next_fmt!("{}"),
      print_error_as!(&str),
      print_complete!(),
    );

    sbj.next(1);
    sbj.next(2);
    sbj.error(RxError::from_error("ERR!"));
  }
}
