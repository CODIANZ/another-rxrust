use crate::all::*;

#[derive(Clone)]
pub struct Observer<Item> {
  pub next: RxFn<dyn FnMut(Item)>,
  pub error: RxFn<dyn FnMut(RxError)>,
  pub complete: RxFn<dyn FnMut()>,
}
