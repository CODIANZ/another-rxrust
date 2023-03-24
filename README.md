# another-rxrust

Unlike `RxRust`, an implementation that maintains the flexibility of `Rx` at the expense of memory and speed efficiency.

```rust
use another_rxrust::prelude::*;
use anyhow::anyhow;
use std::sync::Arc;

fn main() {
  let o = Observable::<i32>::create(|s, _| {
    for n in 1..10 {
      s.next(n);
    }
    s.complete();
  });

  let oo = o.clone();

  let _sbsc = oo
    .flat_map(move |x| match x {
      1 => observables::just(2).map(|x| format!("str {}", x)),
      2 => Observable::create(move |s, _| {
        s.next(x * 10);
        s.next(x * 11);
        s.complete();
      })
      .map(|x| format!("str {}", x)),
      3 => o.clone().map(|x| 1000 + x).map(|x| format!("str {}", x)),
      4 => observables::just(x)
        .map(|x| x * 10)
        .map(|x| format!("str {}", x)),
      5 => observables::empty(),
      9 => observables::error(Arc::new(anyhow!("it's nine!!"))),
      _ => observables::never(),
    })
    .subscribe(
      |x| {
        println!("next {}", x);
      },
      |e| {
        println!("error {:}", e);
      },
      || {
        println!("complete");
      },
    );
}
// next str 2
// next str 20
// next str 22
// next str 1001
// next str 1002
// next str 1003
// next str 1004
// next str 1005
// next str 1006
// next str 1007
// next str 1008
// next str 1009
// next str 40
// error it's nine!!
```

## Implementation status

### observable

- Observable
- just
- never
- error
- empty

### operator

- map
- flat_map
- on_error_resume_next

### subject

- Subject
- BehaviorSubject
