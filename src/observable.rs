use std::time::Duration;

use crate::internals::function_wrapper::*;
use crate::prelude::*;
use scheduler::IScheduler;

#[derive(Clone)]
pub struct Subscription<'a> {
  fn_unsubscribe: FunctionWrapper<'a, (), ()>,
}

impl<'a> Subscription<'a> {
  pub fn new<Unsub>(unsub: Unsub) -> Subscription<'a>
  where
    Unsub: Fn() + Send + Sync + 'a,
  {
    Subscription {
      fn_unsubscribe: FunctionWrapper::new(move |_| unsub()),
    }
  }
  pub fn unsubscribe(&self) {
    self.fn_unsubscribe.call_and_clear_if_available(());
  }
}

#[derive(Clone)]
pub struct Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  source: FunctionWrapper<'a, Observer<'a, Item>, ()>,
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync,
{
  pub fn create<Source>(source: Source) -> Observable<'a, Item>
  where
    Source: Fn(Observer<'a, Item>) + Send + Sync + 'a,
  {
    Observable {
      source: FunctionWrapper::new(source),
    }
  }

  pub(crate) fn inner_subscribe(&self, observer: Observer<'a, Item>) -> Subscription<'a> {
    let unsub_observer = observer.clone();
    self.source.call(observer.clone());
    Subscription::new(move || {
      unsub_observer.unsubscribe();
    })
  }

  pub fn subscribe<Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Subscription<'a>
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    self.inner_subscribe(Observer::new(next, error, complete))
  }

  pub fn map<Out, F>(&self, f: F) -> Observable<'a, Out>
  where
    F: Fn(Item) -> Out + Send + Sync + 'a,
    Out: Clone + Send + Sync,
  {
    operators::MapOp::new(f).execute(self.clone())
  }

  pub fn flat_map<Out, F>(&self, f: F) -> Observable<'a, Out>
  where
    F: Fn(Item) -> Observable<'a, Out> + Send + Sync + 'a,
    Out: Clone + Send + Sync,
  {
    operators::FlatMapOp::new(f).execute(self.clone())
  }

  pub fn on_error_resume_next<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(RxError) -> Observable<'a, Item> + Send + Sync + 'a,
  {
    operators::OnErrorResumeNextOp::new(f).execute(self.clone())
  }

  pub fn observe_on<Scheduler, SchedulerCreator>(
    &self,
    scheduler_ctor: SchedulerCreator,
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    operators::ObserveOnOp::new(scheduler_ctor).execute(self.clone())
  }

  pub fn subscribe_on<Scheduler, SchedulerCreator>(
    &self,
    scheduler_ctor: SchedulerCreator,
  ) -> Observable<'a, Item>
  where
    Scheduler: IScheduler<'a> + Clone + Send + Sync + 'a,
    SchedulerCreator: Fn() -> Scheduler + Send + Sync + 'a,
  {
    operators::SubscribeOnOp::new(scheduler_ctor).execute(self.clone())
  }

  pub fn take(&self, count: usize) -> Observable<'a, Item> {
    operators::TakeOp::new(count).execute(self.clone())
  }

  pub fn take_last(&self, count: usize) -> Observable<'a, Item> {
    operators::TakeLastOp::new(count).execute(self.clone())
  }

  pub fn first(&self) -> Observable<'a, Item> {
    operators::FirstOp::new().execute(self.clone())
  }

  pub fn last(&self) -> Observable<'a, Item> {
    operators::LastOp::new().execute(self.clone())
  }

  pub fn skip(&self, count: usize) -> Observable<'a, Item> {
    operators::SkipOp::new(count).execute(self.clone())
  }

  pub fn skip_last(&self, count: usize) -> Observable<'a, Item> {
    operators::SkipLastOp::new(count).execute(self.clone())
  }

  pub fn ignore_elements(&self) -> Observable<'a, Item> {
    operators::IgnoreElementsOp::new().execute(self.clone())
  }

  pub fn take_until<TriggerValue>(
    &self,
    trigger: Observable<'a, TriggerValue>,
  ) -> Observable<'a, Item>
  where
    TriggerValue: Clone + Send + Sync,
  {
    operators::TakeUntilOp::new(trigger).execute(self.clone())
  }

  pub fn skip_until<TriggerValue>(
    &self,
    trigger: Observable<'a, TriggerValue>,
  ) -> Observable<'a, Item>
  where
    TriggerValue: Clone + Send + Sync,
  {
    operators::SkipUntilOp::new(trigger).execute(self.clone())
  }

  pub fn take_while<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    operators::TakeWhileOp::new(f).execute(self.clone())
  }

  pub fn skip_while<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    operators::SkipWhileOp::new(f).execute(self.clone())
  }

  pub fn filter<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(Item) -> bool + Send + Sync + 'a,
  {
    operators::FilterOp::new(f).execute(self.clone())
  }

  pub fn sample<TriggerValue>(&self, trigger: Observable<'a, TriggerValue>) -> Observable<'a, Item>
  where
    TriggerValue: Clone + Send + Sync,
  {
    operators::SampleOp::new(trigger).execute(self.clone())
  }

  pub fn element_at(&self, count: usize) -> Observable<'a, Item> {
    operators::ElementAtOp::new(count).execute(self.clone())
  }

  pub fn tap<Next, Error, Complete>(
    &self,
    next: Next,
    error: Error,
    complete: Complete,
  ) -> Observable<'a, Item>
  where
    Next: Fn(Item) + Send + Sync + 'a,
    Error: Fn(RxError) + Send + Sync + 'a,
    Complete: Fn() + Send + Sync + 'a,
  {
    operators::TapOp::new(next, error, complete).execute(self.clone())
  }

  pub fn retry(&self, max_retry: usize) -> Observable<'a, Item> {
    operators::RetryOp::new(max_retry).execute(self.clone())
  }

  pub fn retry_when<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn(RxError) -> bool + Send + Sync + 'a,
  {
    operators::RetryWhenOp::new(f).execute(self.clone())
  }

  pub fn merge(&self, observables: &[Observable<'a, Item>]) -> Observable<'a, Item> {
    operators::MergeOp::new(observables).execute(self.clone())
  }

  pub fn amb(&self, observables: &[Observable<'a, Item>]) -> Observable<'a, Item> {
    operators::AmbOp::new(observables).execute(self.clone())
  }

  pub fn zip(&self, observables: &[Observable<'a, Item>]) -> Observable<'a, Vec<Item>> {
    operators::ZipOp::new(observables).execute(self.clone())
  }

  pub fn combine_latest<Out, F>(
    &self,
    observables: &[Observable<'a, Item>],
    f: F,
  ) -> Observable<'a, Out>
  where
    Out: Clone + Send + Sync,
    F: Fn(Vec<Item>) -> Out + Send + Sync + 'a,
  {
    operators::CombineLatestOp::new(observables, f).execute(self.clone())
  }

  pub fn pipe<F, Out>(&self, f: F) -> Observable<'a, Out>
  where
    Out: Clone + Send + Sync,
    F: Fn(Observable<Item>) -> Observable<Out>,
  {
    f(self.clone())
  }

  pub fn reduce<F>(&self, f: F) -> Observable<'a, Item>
  where
    F: Fn((Item, Item)) -> Item + Send + Sync + 'a,
  {
    operators::ReduceOp::new(f).execute(self.clone())
  }

  pub fn start_with<Iter>(&self, iter: Iter) -> Observable<'a, Item>
  where
    Iter: Iterator<Item = Item> + Clone + Send + Sync + 'a,
  {
    operators::StartWithOp::new(iter).execute(self.clone())
  }

  pub fn count(&self) -> Observable<'a, usize> {
    operators::CountOp::new().execute(self.clone())
  }

  pub fn delay(&self, dur: Duration) -> Observable<'a, Item> {
    operators::DelayOp::new(dur).execute(self.clone())
  }

  pub fn publish(&self) -> publish::PublishOp<'a, Item> {
    operators::PublishOp::new(self.clone())
  }

  pub fn ref_count(&self) -> ref_count::RefCountOp<'a, Item> {
    operators::RefCountOp::new(self.clone())
  }

  pub fn default_if_empty(&self, target: Item) -> Observable<'a, Item> {
    operators::DefaultIfEmpty::new(target).execute(self.clone())
  }

  pub fn switch_on_next(&self, target: Observable<'a, Item>) -> Observable<'a, Item> {
    operators::SwitchOnNextOp::new(target).execute(self.clone())
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + PartialEq,
{
  pub fn distinct_until_changed(&self) -> Observable<'a, Item> {
    operators::DistinctUntilChangedOp::new().execute(self.clone())
  }
  pub fn contains(&self, target: Item) -> Observable<'a, bool> {
    operators::ContainsOp::new(target).execute(self.clone())
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + PartialOrd,
{
  pub fn min(&self) -> Observable<'a, Item> {
    operators::MinOp::new().execute(self.clone())
  }

  pub fn max(&self) -> Observable<'a, Item> {
    operators::MaxOp::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;
  use crate::tests::common::*;
  use std::{thread, time};

  #[test]
  fn basic() {
    let o = Observable::create(|s| {
      for n in 0..10 {
        s.next(n);
      }
      s.complete();
    });

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
  }

  #[test]
  fn thread() {
    let o = Observable::create(|s| {
      thread::spawn(move || {
        for n in 0..100 {
          if !s.is_subscribed() {
            break;
          }
          s.next(n);
        }
        if s.is_subscribed() {
          s.complete();
        }
      });
    });

    o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
    println!("started");
  }

  #[test]
  fn unsubscribe() {
    let o = Observable::create(|s| {
      thread::spawn(move || {
        for n in 0..100 {
          if !s.is_subscribed() {
            println!("break!");
            break;
          }
          s.next(n);
          thread::sleep(time::Duration::from_millis(100));
        }
        if s.is_subscribed() {
          s.complete();
        }
      });
    });

    let sbsc = o.subscribe(
      |x| println!("next {}", x),
      |e| println!("error {:}", error_to_string(&e)),
      || println!("complete"),
    );
    println!("started");
    thread::sleep(time::Duration::from_millis(1000));
    sbsc.unsubscribe();
    thread::sleep(time::Duration::from_millis(1000));
  }

  #[test]
  fn pipe() {
    observables::just(1)
      .pipe(|s| operators::MapOp::new(|x| x * 2).execute(s))
      .subscribe(
        |x| println!("next {}", x),
        |e| println!("error {:}", error_to_string(&e)),
        || println!("complete"),
      );
  }
}
