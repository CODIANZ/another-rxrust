## Implementation Status

Quoted from [ReactiveX](https://reactivex.io/).

### Creating Observables

Operators that originate new Observables.

- [x] [Create](https://reactivex.io/documentation/operators/create.html) — create an Observable from scratch by calling observer methods programmatically
- [x] [Defer](https://reactivex.io/documentation/operators/defer.html) — do not create the Observable until the observer subscribes, and create a fresh Observable for each observer
- [x] [Empty/Never/Throw](https://reactivex.io/documentation/operators/empty-never-throw.html) — create Observables that have very precise and limited behavior
  - Throw - `error`
- [x] [From](https://reactivex.io/documentation/operators/from.html) — convert some other object or data structure into an Observable
  - `from_iter`
- [x] [Interval](https://reactivex.io/documentation/operators/interval.html) — create an Observable that emits a sequence of integers spaced by a particular time interval
- [x] [Just](https://reactivex.io/documentation/operators/just.html) — convert an object or a set of objects into an Observable that emits that or those objects
- [x] [Range](https://reactivex.io/documentation/operators/range.html) — create an Observable that emits a range of sequential integers
- [x] [Repeat](https://reactivex.io/documentation/operators/repeat.html) — create an Observable that emits a particular item or sequence of items repeatedly
- [x] [Start](https://reactivex.io/documentation/operators/start.html) — create an Observable that emits the return value of a function
- [x] [Timer](https://reactivex.io/documentation/operators/timer.html) — create an Observable that emits a single item after a given delay

### Transforming Observables

Operators that transform items that are emitted by an Observable.

- [x] [Buffer](https://reactivex.io/documentation/operators/buffer.html) — periodically gather items from an Observable into bundles and emit these bundles rather than emitting the items one at a time
  - `buffer_with_count`
- [x] [FlatMap](https://reactivex.io/documentation/operators/flatmap.html) — transform the items emitted by an Observable into Observables, then flatten the emissions from those into a single Observable
- [ ] [GroupBy](https://reactivex.io/documentation/operators/groupby.html) — divide an Observable into a set of Observables that each emit a different group of items from the original Observable, organized by key
- [x] [Map](https://reactivex.io/documentation/operators/map.html) — transform the items emitted by an Observable by applying a function to each item
- [ ] [Scan](https://reactivex.io/documentation/operators/scan.html) — apply a function to each item emitted by an Observable, sequentially, and emit each successive value
- [ ] [Window](https://reactivex.io/documentation/operators/window.html) — periodically subdivide items from an Observable into Observable windows and emit these windows rather than emitting the items one at a time

### Filtering Observables

Operators that selectively emit items from a source Observable.

- [x] [Debounce](https://reactivex.io/documentation/operators/debounce.html) — only emit an item from an Observable if a particular timespan has passed without it emitting another item
- [x] [Distinct](https://reactivex.io/documentation/operators/distinct.html) — suppress duplicate items emitted by an Observable
  - distinct_until_changed
- [x] [ElementAt](https://reactivex.io/documentation/operators/elementat.html) — emit only item n emitted by an Observable
- [x] [Filter](https://reactivex.io/documentation/operators/filter.html) — emit only those items from an Observable that pass a predicate test
- [x] [First](https://reactivex.io/documentation/operators/first.html) — emit only the first item, or the first item that meets a condition, from an Observable
- [x] [IgnoreElements](https://reactivex.io/documentation/operators/ignoreelements.html) — do not emit any items from an Observable but mirror its termination notification
- [x] [Last](https://reactivex.io/documentation/operators/last.html) — emit only the last item emitted by an Observable
- [x] [Sample](https://reactivex.io/documentation/operators/sample.html) — emit the most recent item emitted by an Observable within periodic time intervals
- [x] [Skip](https://reactivex.io/documentation/operators/skip.html) — suppress the first n items emitted by an Observable
- [x] [SkipLast](https://reactivex.io/documentation/operators/skiplast.html) — suppress the last n items emitted by an Observable
- [x] [Take](https://reactivex.io/documentation/operators/take.html) — emit only the first n items emitted by an Observable
- [x] [TakeLast](https://reactivex.io/documentation/operators/takelast.html) — emit only the last n items emitted by an Observable

### Combining Observables

Operators that work with multiple source Observables to create a single Observable

- [ ] [And/Then/When](https://reactivex.io/documentation/operators/and-then-when.html) — combine sets of items emitted by two or more Observables by means of Pattern and Plan intermediaries
- [x] [CombineLatest](https://reactivex.io/documentation/operators/combinelatest.html) — when an item is emitted by either of two Observables, combine the latest item emitted by each Observable via a specified function and emit items based on the results of this function
- [ ] [Join](https://reactivex.io/documentation/operators/join.html) — combine items emitted by two Observables whenever an item from one Observable is emitted during a time window defined according to an item emitted by the other Observable
- [x] [Merge](https://reactivex.io/documentation/operators/merge.html) — combine multiple Observables into one by merging their emissions
- [x] [StartWith](https://reactivex.io/documentation/operators/startwith.html) — emit a specified sequence of items before beginning to emit the items from the source Observable
- [x] [Switch](https://reactivex.io/documentation/operators/switch.html) — convert an Observable that emits Observables into a single Observable that emits the items emitted by the most-recently-emitted of those Observables
  - `switch_on_next`
- [x] [Zip](https://reactivex.io/documentation/operators/zip.html) — combine the emissions of multiple Observables together via a specified function and emit single items for each combination based on the results of this function

### Error Handling Operators

Operators that help to recover from error notifications from an Observable

- [x] [Catch](https://reactivex.io/documentation/operators/catch.html) — recover from an onError notification by continuing the sequence without error
  - `on_error_resume_next`
- [x] [Retry](https://reactivex.io/documentation/operators/retry.html) — if a source Observable sends an onError notification, resubscribe to it in the hopes that it will complete without error
  - `retry`
  - `retry_when`

### Observable Utility Operators

A toolbox of useful Operators for working with Observables

- [x] [Delay](https://reactivex.io/documentation/operators/delay.html) — shift the emissions from an Observable forward in time by a particular amount
- [x] [Do](https://reactivex.io/documentation/operators/do.html) — register an action to take upon a variety of Observable lifecycle events
  - `tap`
- [ ] [Materialize/Dematerialize](https://reactivex.io/documentation/operators/materialize-dematerialize.html) — represent both the items emitted and the notifications sent as emitted items, or reverse this process
- [x] [ObserveOn](https://reactivex.io/documentation/operators/observeon.html) — specify the scheduler on which an observer will observe this Observable
- [ ] [Serialize](https://reactivex.io/documentation/operators/serialize.html) — force an Observable to make serialized calls and to be well-behaved
- [x] [Subscribe](https://reactivex.io/documentation/operators/subscribe.html) — operate upon the emissions and notifications from an Observable
- [x] [SubscribeOn](https://reactivex.io/documentation/operators/subscribeon.html) — specify the scheduler an Observable should use when it is subscribed to
- [x] [TimeInterval](https://reactivex.io/documentation/operators/timeinterval.html) — convert an Observable that emits items into one that emits indications of the amount of time elapsed between those emissions
- [x] [Timeout](https://reactivex.io/documentation/operators/timeout.html) — mirror the source Observable, but issue an error notification if a particular period of time elapses without any emitted items
- [x] [Timestamp](https://reactivex.io/documentation/operators/timestamp.html) — attach a timestamp to each item emitted by an Observable
- [x] [Using](https://reactivex.io/documentation/operators/using.html) — create a disposable resource that has the same lifespan as the Observable
  - `utils::Using`

### Conditional and Boolean Operators

Operators that evaluate one or more Observables or items emitted by Observables

- [ ] [All](https://reactivex.io/documentation/operators/all.html) — determine whether all items emitted by an Observable meet some criteria
- [x] [Amb](https://reactivex.io/documentation/operators/amb.html) — given two or more source Observables, emit all of the items from only the first of these Observables to emit an item
- [x] [Contains](https://reactivex.io/documentation/operators/contains.html) — determine whether an Observable emits a particular item or not
- [x] [DefaultIfEmpty](https://reactivex.io/documentation/operators/defaultifempty.html) — emit items from the source Observable, or a default item if the source Observable emits nothing
- [ ] [SequenceEqual](https://reactivex.io/documentation/operators/sequenceequal.html) — determine whether two Observables emit the same sequence of items
- [x] [SkipUntil](https://reactivex.io/documentation/operators/skipuntil.html) — discard items emitted by an Observable until a second Observable emits an item
- [x] [SkipWhile](https://reactivex.io/documentation/operators/skipwhile.html) — discard items emitted by an Observable until a specified condition becomes false
- [x] [TakeUntil](https://reactivex.io/documentation/operators/takeuntil.html) — discard items emitted by an Observable after a second Observable emits an item or terminates
- [x] [TakeWhile](https://reactivex.io/documentation/operators/takewhile.html) — discard items emitted by an Observable after a specified condition becomes false

### Mathematical and Aggregate Operators

Operators that operate on the entire sequence of items emitted by an Observable

- [ ] [Average](https://reactivex.io/documentation/operators/average.html) — calculates the average of numbers emitted by an Observable and emits this average
- [ ] [Concat](https://reactivex.io/documentation/operators/concat.html) — emit the emissions from two or more Observables without interleaving them
- [x] [Count](https://reactivex.io/documentation/operators/count.html) — count the number of items emitted by the source Observable and emit only this value
- [x] [Max](https://reactivex.io/documentation/operators/max.html) — determine, and emit, the maximum-valued item emitted by an Observable
- [x] [Min](https://reactivex.io/documentation/operators/min.html) — determine, and emit, the minimum-valued item emitted by an Observable
- [x] [Reduce](https://reactivex.io/documentation/operators/reduce.html) — apply a function to each item emitted by an Observable, sequentially, and emit the final value
- [ ] [Sum](https://reactivex.io/documentation/operators/sum.html) — calculate the sum of numbers emitted by an Observable and emit this sum

### Connectable Observable Operators

Specialty Observables that have more precisely-controlled subscription dynamics

- [x] [Connect](https://reactivex.io/documentation/operators/connect.html) — instruct a connectable Observable to begin emitting items to its subscribers
- [x] [Publish](https://reactivex.io/documentation/operators/publish.html) — convert an ordinary Observable into a connectable Observable
- [x] [RefCount](https://reactivex.io/documentation/operators/refcount.html) — make a Connectable Observable behave like an ordinary Observable
- [ ] [Replay](https://reactivex.io/documentation/operators/replay.html) — ensure that all observers see the same sequence of emitted items, even if they subscribe after the Observable has begun emitting items

### Operators to Convert Observables

- [ ] [To](https://reactivex.io/documentation/operators/to.html) — convert an Observable into another object or data structure

### Subjects

- [x] [AsyncSubject](https://reactivex.io/documentation/subject.html) - emit the last value on completion
- [x] [BehaviorSubject](https://reactivex.io/documentation/subject.html) - emit the current value to new subscribers
- [x] [PublishSubject](https://reactivex.io/documentation/subject.html) - emit all subsequently observed items to the subscriber
  - `subjects::Subject`
- [ ] [ReplaySubject](https://reactivex.io/documentation/subject.html) - emit old values to new subscribers

### Schedulers

- [x] [Default](https://reactivex.io/documentation/scheduler.html) - scheduler to run on the current thread.
- [x] [NewThread](https://reactivex.io/documentation/scheduler.html) - scheduler that creates a new thread and executes it there.

## Utilities

### utils

- `ready_set_go` - Composing Observables and Functions

### subscribe macros

- `junk_next!()` - discard the next
- `junk_error!()` - discard the error
- `junk_complete!()` - discard the complete
- `print_next!()` - print the next
- `print_error!()` - print the error
- `print_complete!()` - print the complete
- `print_next_fmt!()` - print the next with format
- `panic_error!()` - panic if error
