# with_lock
Deadlock freedom

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
    println!("{:?}", *a_lock + *b_lock);
    let a_lock_2 = a.lock().unwrap();
    let b_lock_2 = b.lock().unwrap();
    println!("{:?}", *a_lock_2 + *b_lock_2);
}
```
That code will log `5` once, when it should log twice. As you can see here, it is deadlocking.

However, we can prevent this by replacing our manual calls of `.lock` with `.with_lock`. Code that wouldn't error would look something like:

```rust
use std::sync::Mutex;
use with_lock::WithLock;

fn main() {
    let a = WithLock::<i64>::new(Mutex::new(2));
    let b = WithLock::<i64>::new(Mutex::new(3));
    let a_lock = a.with_lock(|s| *s);
    let b_lock = b.with_lock(|s| *s);
    println!("{:?}", a_lock + b_lock);
    let a_lock_2 = a.with_lock(|s| *s);
    let b_lock_2 = b.with_lock(|s| *s);
    println!("{:?}", a_lock_2 + b_lock_2);
}
```

That code would log `5` twice. This is an example of how it can prevent deadlocks.

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
    println!("{:?}", *a_lock + *b_lock);
    let a_lock_2 = a.lock().unwrap();
    let b_lock_2 = b.lock().unwrap();
    println!("{:?}", *a_lock_2 + *b_lock_2);
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
    println!("{:?}", a_locked + b_locked);
    let a_lock_2 = a.get();
    let b_lock_2 = b.get();
    println!("{:?}", a_lock_2 + b_lock_2);
}
```