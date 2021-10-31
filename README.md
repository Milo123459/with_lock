# with_lock
Prevent deadlocks

## Example

Say you have this code:

```rs
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

However, we can prevent this by replacing our manual calls of `.lock` with `.with_lock`. So code that wouldn't error would look something like:

```rs
use std::sync::Mutex;
use with_lock::WithLock;

fn main() {
    let a = WithLock::<i64>::new(2);
    let b = WithLock::<i64>::new(3);
    let a_lock = a.with_lock(|s| *s);
    let b_lock = b.with_lock(|s| *s);
    println!("{:?}", *a_lock + *b_lock);
    let a_lock_2 = a.with_lock(|s| *s);
    let b_lock_2 = b.with_lock(|s| *s);
    println!("{:?}", *a_lock_2 + *b_lock_2);
}
```

That code would log `5` twice. This is an example of how it can prevent deadlocks.