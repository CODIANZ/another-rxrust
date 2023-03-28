pub trait IScheduler<'a> {
  fn post<F>(&self, f: F)
  where
    F: Fn() + Clone + Send + Sync + 'a;
}
