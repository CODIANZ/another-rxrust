# another-rxrust

## Why not `rxRust`?

`rxRust` is for streaming single data, not for complex streaming of `ReactiveX`. Also, there is no error recovery, and it is different from general `ReactiveX`.
Therefore, something that can be written easily with `another-rxrust` as shown below becomes extremely difficult with `rxRust`.

```rust
use crate::prelude::*;
use anyhow::anyhow;
use std::{thread, time};

fn main() {
  fn ob() -> Observable<i32> {
    Observable::create(|s| {
      s.next(1);
      s.next(2);
      s.next(3);
      s.next(4);
      s.complete();
      Subscription::new(|| {})
    })
  }

  ob()
    .observe_on(schedulers::new_thread())
    .flat_map(|x| match x {
      1 => observables::empty(),
      2 => observables::just(x),
      3 => ob().map(|x| (x + 100)),
      4 => observables::error(RxError::new(anyhow!("err"))),
      _ => observables::never(),
    })
    .map(|x| format!("{}", x))
    .on_error_resume_next(|e| ob().map(move |x| format!("resume {:} {}", e.error, x)))
    .subscribe(
      |x| {
        println!("next {}", x);
      },
      |e| {
        println!("error {:}", e.error);
      },
      || {
        println!("complete");
      },
    );

  thread::sleep(time::Duration::from_millis(500));
}
// next 2
// next 101
// next 102
// next 103
// next 104
// next resume err 1
// next resume err 2
// next resume err 3
// next resume err 4
// complete
```

## Implementation policy

Based on the problems of `rxRust`, `another-rxrust` has the following implementation policy.

- It is assumed that the values and functions that can be emitted may be shared between threads.
- Only `Clone + Send + Sync + 'static` can be issued.
- Functions should be `Fn() + Send + Sync + 'static` only.
- Errors use `anyhow::Error`.
- Prioritize flexibility over memory efficiency and execution speed.

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
- observe_on

### subject

- Subject
- BehaviorSubject

### scheduler

- new_thread
