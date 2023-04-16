use crate::internals::function_wrapper::*;

#[derive(Clone)]
pub struct Subscription<'a> {
  fn_unsubscribe: FunctionWrapper<'a, (), ()>,
  fn_is_subscribed: FunctionWrapper<'a, (), bool>,
}

impl<'a> Subscription<'a> {
  pub fn new<Unsub, Issub>(unsub: Unsub, issub: Issub) -> Subscription<'a>
  where
    Unsub: Fn() + Send + Sync + 'a,
    Issub: Fn() -> bool + Send + Sync + 'a,
  {
    Subscription {
      fn_unsubscribe: FunctionWrapper::new(move |_| unsub()),
      fn_is_subscribed: FunctionWrapper::new(move |_| issub()),
    }
  }
  pub fn unsubscribe(&self) {
    self.fn_unsubscribe.call_and_clear_if_available(());
  }
  pub fn is_subscribed(&self) -> bool {
    if let Some(x) = self.fn_is_subscribed.call_if_available(()) {
      x
    } else {
      false
    }
  }
}
