use crate::all::*;

#[derive(Clone)]
pub struct Subscriber<Item> {
  pub(crate) observer: Option<Observer<Item>>,
}

impl<Item> Subscriber<Item> {
  pub fn next(&self, x: Item) {
    if let Some(ref o) = self.observer {
      // RefCellで囲んだFnMutはDerefMutが実装されていない
      // https://takoyaking.hatenablog.com/entry/refcell_fnmut_deref
      (&mut *o.next.borrow_mut())(x);
    }
  }
  pub fn error(&mut self, x: RxError) {
    if let Some(ref o) = self.observer {
      (&mut *o.error.borrow_mut())(x);
      self.discard();
    }
  }
  pub fn complete(&mut self) {
    if let Some(ref o) = self.observer {
      (&mut *o.complete.borrow_mut())();
      self.discard();
    }
  }
  pub fn discard(&mut self) {
    self.observer = None;
  }
}
