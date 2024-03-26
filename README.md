# another-rxrust

## why new implementation?

`rxRust` is a `Rust` language implementation of `ReactiveX`, but it is not suitable for complex combinations of `observable`.
For those who use `ReactiveX` in other languages such as `rxjs(TypeScript)` or `rxcpp`, `rxRust` is a very difficult library.

Therefore, I created `another-rxrust`, thinking that I needed a library that allows `observable` to be connected in the same way as other platforms, and that `ReactiveX` can be enjoyed in `Rust`.

In addition, `ReactiveX` may not be the best solution if the purpose is to parallelize heavy processing and speed it up. However, `ReactiveX` is one answer for complex combinations of non-blocking I/O and error handling.

## Implementation policy

- It is assumed that the values and functions that can be emitted may be shared between threads.
- Value to emit should be `Clone + Send + Sync` only.
- Use `move` to emit values ​​as much as possible.
- Functions should be `Fn() + Send + Sync` only.
- Errors are type-erased using `std::any`.
  - Thus requiring a `'static` lifetime.
  - You can use `anyhow`.
- Prioritize flexibility over memory efficiency and execution speed.

## Implementation status

See [implementation status](implementation_status.md).

## Installation

```sh
cargo add  another-rxrust
```

or

```toml
[dependencies]
another-rxrust = {}
```

If you want to use it on the web, set it as follows.

```toml
[dependencies]
another-rxrust = {features=["web"]}
```

## Samples

### from_iter, map, zip

```rust
use another_rxrust::prelude::*;
use another_rxrust::{print_next_fmt, print_error, print_complete};

fn main() {
  let ob = observables::from_iter(0..10);
  ob.zip(&[ob.map(|x| x + 10), ob.map(|x| x + 20)]).subscribe(
    print_next_fmt!("{:?}"),
    print_error!(),
    print_complete!(),
  );
}
// [console-results]
// next - [0, 10, 20]
// next - [1, 11, 21]
// next - [2, 12, 22]
// next - [3, 13, 23]
// next - [4, 14, 24]
// next - [5, 15, 25]
// next - [6, 16, 26]
// next - [7, 17, 27]
// next - [8, 18, 28]
// next - [9, 19, 29]
// complete
```

### subject, scheduler, take, sample

```rust
use another_rxrust::prelude::*;
use std::{thread, time};
use another_rxrust::{print_next_fmt, print_error, print_complete};

fn main() {
  let sbj = subjects::Subject::new();
  observables::interval(
    time::Duration::from_millis(100),
    schedulers::new_thread_scheduler(),
  )
  .sample(sbj.observable())
  .take(3)
  .subscribe(print_next_fmt!("{}"), print_error!(), print_complete!());

  (0..3).for_each(|_| {
    thread::sleep(time::Duration::from_millis(500));
    sbj.next(());
  });
  sbj.complete();
  thread::sleep(time::Duration::from_millis(500));
}
// [console-results (Depends on execution environment)]
// next - 3
// next - 8
// next - 13
// complete
```

### just, error, emptry, never, flat_map...

```rust
use another_rxrust::prelude::*;
use std::{thread, time};

fn main() {
  // observable creator function
  fn ob() -> Observable<'static, i32> {
    Observable::create(|s| {
      s.next(100);
      s.next(200);
      s.complete();
    })
  }

  observables::from_iter(1..10)
    .observe_on(schedulers::new_thread_scheduler())
    .flat_map(|x| match x {
      1 => observables::empty(),
      2 => observables::just(x),
      3 => ob().map(move |y| (y + x)),
      4 => observables::error(RxError::from_error("some error")),
      _ => observables::never(),
    })
    .map(|x| format!("{}", x))
    .on_error_resume_next(|e| {
      ob().map(move |x| {
        format!(
          "resume {:?} {}",
          e.downcast_ref::<&str>(),
          x
        )
      })
    })
    .subscribe(
      |x| println!("{}", x),
      |e| println!("{:?}", e.downcast_ref::<&str>()),
      || println!("complete"),
    );

  thread::sleep(time::Duration::from_millis(500));
}
// [console-results]
// next 2
// next 103
// next 203
// next resume "some error" 100
// next resume "some error" 200
// complete
```
