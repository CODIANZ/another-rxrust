# another-rxrust

## Why not `rxRust`?

`rxRust` is for streaming single data, not for complex streaming of `ReactiveX`. Also, there is no error recovery, and it is different from general `ReactiveX`.
Therefore, something that can be written easily with `another-rxrust` as shown below becomes extremely difficult with `rxRust`.

```rust
use crate::prelude::*;
use anyhow::anyhow;
use std::{thread, time};

fn main() {
  fn ob() -> Observable<'static, i32> {
    Observable::create(|s| {
      s.next(100);
      s.next(200);
      s.complete();
    })
  }

  observables::from_iter(vec![1, 2, 3, 4, 5].into_iter())
    .observe_on(schedulers::new_thread_scheduler())
    .flat_map(|x| match x {
      1 => observables::empty(),
      2 => observables::just(x),
      3 => ob().map(move |y| (y + x)),
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

  thread::sleep(time::Duration::from_millis(600));
}

// next 2
// next 103
// next 203
// next resume err 100
// next resume err 200
// complete
```

## Implementation policy

Based on the problems of `rxRust`, `another-rxrust` has the following implementation policy.

- It is assumed that the values and functions that can be emitted may be shared between threads.
- Value to emit should be `Clone + Send + Sync` only.
- Use `move` to emit values ​​as much as possible.
- Functions should be `Fn() + Send + Sync` only.
- Default errors use `std::any`. If `features=["anyhow"]` use `anyhow::Error`.
- Prioritize flexibility over memory efficiency and execution speed.

## Implementation status

### observable

- Observable
- just
- never
- error
- empty
- from_iter
- timer

### operator

- map
- flat_map
- on_error_resume_next
- observe_on
- subscribe_on
- skip
- skip_last
- skip_until
- skip_while
- take
- take_last
- take_until
- take_while
- tap
- distinct_until_changed
- retry
- retry_when

### subject

- Subject
- BehaviorSubject

### scheduler

- new_thread_scheduler
- default_scheduler
