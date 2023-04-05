use crate::prelude::*;
use std::{
  future::Future,
  marker::PhantomData,
  sync::{Arc, RwLock},
  task::{self, Waker},
};

#[derive(Clone)]
pub struct ToVec<'a, Item>
where
  Item: Clone + Send + Sync,
{
  buffer: Arc<RwLock<Vec<Item>>>,
  done: Arc<RwLock<bool>>,
  err: Arc<RwLock<Option<RxError>>>,
  waker: Arc<RwLock<Option<Waker>>>,
  _lifetime: PhantomData<&'a ()>,
}

impl<'a, Item> ToVec<'a, Item>
where
  Item: Clone + Send + Sync,
{
  fn start(&self, source: Observable<'a, Item>) {
    let buff_next = Arc::clone(&self.buffer);
    let done_error = Arc::clone(&self.done);
    let done_complete = Arc::clone(&self.done);
    let err_error = Arc::clone(&self.err);
    let waker_error = Arc::clone(&self.waker);
    let waker_complete = Arc::clone(&self.waker);
    source.subscribe(
      move |x| buff_next.write().unwrap().push(x),
      move |e| {
        *err_error.write().unwrap() = Some(e);
        *done_error.write().unwrap() = true;
        if let Some(w) = waker_error.read().unwrap().clone() {
          w.wake();
        }
      },
      move || {
        *done_complete.write().unwrap() = true;
        if let Some(w) = waker_complete.read().unwrap().clone() {
          w.wake();
        }
      },
    );
  }
}

impl<'a, Item> Future for ToVec<'a, Item>
where
  Item: Clone + Send + Sync,
{
  type Output = Result<Arc<RwLock<Vec<Item>>>, RxError>;

  fn poll(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    let mut waker = self.waker.write().unwrap();

    if *self.done.read().unwrap() {
      if let Some(err) = &*self.err.read().unwrap() {
        task::Poll::Ready(Err(err.clone()))
      } else {
        task::Poll::Ready(Ok(Arc::clone(&self.buffer)))
      }
    } else {
      *waker = Some(cx.waker().clone());
      task::Poll::Pending
    }
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn to_vec(&self) -> ToVec<Item> {
    let inst = ToVec {
      buffer: Arc::new(RwLock::new(Vec::new())),
      done: Arc::new(RwLock::new(false)),
      err: Arc::new(RwLock::new(None)),
      waker: Arc::new(RwLock::new(None)),
      _lifetime: PhantomData,
    };
    inst.start(self.clone());
    inst
  }
}

#[cfg(all(test, not(feature = "web")))]
mod test {
  use crate::prelude::*;
  use std::time;

  #[tokio::test]
  async fn basic() {
    match observables::just(1).to_vec().await {
      Ok(v) => println!("Ok -> {:?}", v.read().unwrap()),
      Err(e) => println!("Err -> {:?}", e.downcast_ref::<&str>()),
    }
  }

  #[tokio::test]
  async fn thread() {
    match observables::interval(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    )
    .take(5)
    .to_vec()
    .await
    {
      Ok(v) => println!("Ok -> {:?}", v.read().unwrap()),
      Err(e) => println!("Err -> {:?}", e.downcast_ref::<&str>()),
    }
  }

  #[tokio::test]
  async fn error() {
    match observables::interval(
      time::Duration::from_millis(100),
      schedulers::new_thread_scheduler(),
    )
    .take(5)
    .flat_map(|x| {
      if x == 3 {
        observables::error(RxError::from_error("ERR!"))
      } else {
        observables::just(x)
      }
    })
    .to_vec()
    .await
    {
      Ok(v) => println!("Ok -> {:?}", v.read().unwrap()),
      Err(e) => println!("Err -> {:?}", e.downcast_ref::<&str>()),
    }
  }
}
