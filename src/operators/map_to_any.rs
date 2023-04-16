use std::any::Any;
use std::sync::Arc;

use crate::internals::stream_controller::*;
use crate::prelude::*;

#[derive(Clone)]
pub struct MapToAny {}

impl<'a> MapToAny {
  pub fn new() -> MapToAny {
    MapToAny {}
  }
  pub fn execute<Item>(
    &self,
    source: Observable<'a, Item>,
  ) -> Observable<'a, Arc<Box<dyn Any + Send + Sync + 'static>>>
  where
    Item: Clone + Send + Sync + 'static,
  {
    Observable::<Arc<Box<dyn Any + Send + Sync + 'static>>>::create(move |s| {
      let sctl = StreamController::new(s);
      let sctl_next = sctl.clone();
      let sctl_error = sctl.clone();
      let sctl_complete = sctl.clone();

      source.inner_subscribe(sctl.new_observer(
        move |_, x| {
          sctl_next.sink_next(Arc::new(Box::new(x)));
        },
        move |_, e| {
          sctl_error.sink_error(e);
        },
        move |serial| sctl_complete.sink_complete(&serial),
      ));
    })
  }
}

impl<'a, Item> Observable<'a, Item>
where
  Item: Clone + Send + Sync + 'static,
{
  pub fn map_to_any(
    &self,
  ) -> Observable<'a, Arc<Box<dyn Any + Send + Sync + 'static>>> {
    MapToAny::new().execute(self.clone())
  }
}

#[cfg(test)]
mod test {
  use crate::prelude::*;

  #[test]
  fn basic() {
    observables::just(1).map_to_any().subscribe(
      |x| {
        println!("{}", x.downcast_ref::<i32>().unwrap());
      },
      print_error!(),
      print_complete!(),
    );
  }

  #[test]
  fn combination_with_zip() {
    let o = observables::range(0, 10);
    o.map_to_any()
      .zip(&[
        o.map(|x| format!("string {}", x)).map_to_any(),
        o.map(|x| x as f64 * 0.1).map_to_any(),
      ])
      .subscribe(
        |x| {
          println!(
            "{} - {} - {}",
            x[0].downcast_ref::<i64>().unwrap(),
            x[1].downcast_ref::<String>().unwrap(),
            x[2].downcast_ref::<f64>().unwrap(),
          );
        },
        print_error!(),
        print_complete!(),
      );
  }
}
