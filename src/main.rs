use another_rxrust::all::*;
use anyhow::anyhow;
use std::rc::Rc;

fn main() {
  let o = Observable::<i32>::create(rxfn(|mut s, _| {
    for n in 1..10 {
      s.next(n);
    }
    s.complete();
  }));

  let oo = o.clone();

  let _sbsc = oo
    .flat_map(rxfn(move |x| match x {
      1 => observables::just(2).map(rxfn(|x| format!("str {}", x))),
      2 => Observable::create(rxfn(move |mut s, _| {
        s.next(x * 10);
        s.next(x * 11);
        s.complete();
      }))
      .map(rxfn(|x| format!("str {}", x))),
      3 => o
        .clone()
        .map(rxfn(|x| 1000 + x))
        .map(rxfn(|x| format!("str {}", x))),
      4 => observables::just(x)
        .map(rxfn(|x| x * 10))
        .map(rxfn(|x| format!("str {}", x))),
      5 => observables::empty(),
      9 => observables::error(Rc::new(anyhow!("it's nine!!"))),
      _ => observables::never(),
    }))
    .subscribe(
      rxfn(|x| {
        println!("next {}", x);
      }),
      rxfn(|e| {
        println!("error {:}", e);
      }),
      rxfn(|| {
        println!("complete");
      }),
    );
}
