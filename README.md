# with_lock
Deadlock freedom

[Using with_lock? Share it in the discussion!](https://github.com/Milo123459/with_lock/discussions/4)

[Docs](https://docs.rs/with_lock)

## Example

Say you have this code:

```rust,no_run
use std::sync::Mutex;

fn main() {
    let a = Mutex::new(2);
    let b = Mutex::new(3);
    let a_lock = a.lock().unwrap();
    let b_lock = b.lock().unwrap();
    assert_eq!(*a_lock + *b_lock, 5);
    let a_lock_2 = a.lock().unwrap();
    let b_lock_2 = b.lock().unwrap();
    assert_eq!(*a_lock_2 + *b_lock_2, 5);
}
```
That code will run the first `asser_eq!` fine, but the second wouldn't assert due to a deadlock.

However, we can prevent this by replacing our manual calls of `.lock` with `.with_lock`. Code that wouldn't error would look something like:

```rust
use std::sync::Mutex;
use with_lock::WithLock;

fn main() {
    let a = WithLock::<i64>::new(Mutex::new(2));
    let b = WithLock::<i64>::new(Mutex::new(3));
    let a_lock = a.with_lock(|s| *s);
    let b_lock = b.with_lock(|s| *s);
    assert_eq!(a_lock + b_lock, 5);
    let a_lock_2 = a.with_lock(|s| *s);
    let b_lock_2 = b.with_lock(|s| *s);
    assert_eq!(a_lock_2 + b_lock_2, 5);
}
```

This test would pass, and both assertions would be fulfilled. This is an example of how a dead lock was prevented.

## Minimal code changes

For the people that want little to no code changes, `with_lock` exposes a custom MutexCell type.

Code that would produce deadlocks would look like this:

```rust,no_run
use std::sync::Mutex;

fn main() {
    let a = Mutex::new(2);
    let b = Mutex::new(3);
    let a_lock = a.lock().unwrap();
    let b_lock = b.lock().unwrap();
    assert_eq!(*a_lock + *b_lock, 5);
    let a_lock_2 = a.lock().unwrap();
    let b_lock_2 = b.lock().unwrap();
    assert_eq!(*a_lock_2 + *b_lock_2, 5);
}
```

And using the custom MutexCell type that wouldn't deadlock would look like:

```rust
use with_lock::MutexCell;

fn main() {
    let a = MutexCell::new(2);
    let b = MutexCell::new(3);
    let a_locked = a.get();
    let b_locked = b.get();
    assert_eq!(a_locked + b_locked, 5);
    let a_lock_2 = a.get();
    let b_lock_2 = b.get();
    assert_eq!(a_lock_2 + b_lock_2, 5);
}
```

For more examples, see the [examples directory](https://github.com/Milo123459/with_lock/tree/master/examples).
They can be run by cloning this repository and running `cargo run --example <example_name>`.