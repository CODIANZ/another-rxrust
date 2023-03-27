pub trait IScheduler<'a> {
  fn start(&self);
  fn stop(&self);
  fn post<F>(&self, f: F)
  where
    F: Fn() + Clone + Send + Sync + 'a;
}
