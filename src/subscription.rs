use crate::all::*;

#[derive(Clone)]
pub struct Subscription {
  pub(crate) fn_unsubscribe: RxFn<dyn FnMut()>,
}

impl Subscription {
  pub fn unsubscribe(&self) {
    (&mut *self.fn_unsubscribe.borrow_mut())();
  }
}
