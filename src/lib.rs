//! * Sometimes, you are using a sync lock like the standard library's Mutex.
//!  * You need to use it in some async code, and call the `lock` method on the Mutex.
//!  * However, this is prone to deadlocks.
//!  * This crate provides a safe wrapper around the Mutex that will never deadlock.
//!  * It never deadlocks because it has its own function, [`with_lock`], that access' the Mutex, locks it, performs
//!  * the closure, and then drops the lock.
//!  * Enjoy deadlock free code!

use std::sync::Mutex;

pub struct WithLock<T> {
	pub data: Mutex<T>,
}

impl<T> WithLock<T> {
	//! This function gives you access to what Mutex.lock().unwrap() would return.
	//! It will lock the Mutex, perform the closure, and then unlock the Mutex.
	//! This function is safe to use in async code, but can also be used in sync code.
	//! ## Caveats
	//! If you clone the value inside the closure, everything touching that variable will need to be inside the closure.
	//!
	//! ## Example
	//! ```rust
	// use std::sync::Mutex;
	// use with_lock::WithLock;
	//
	// fn main() {
	// let a = WithLock::<i64>::new(2);
	// let b = WithLock::<i64>::new(3);
	// let a_lock = a.with_lock(|s| *s);
	// let b_lock = b.with_lock(|s| *s);
	// assert_eq!(*a_lock + *b_lock, 5);
	// let a_lock_2 = a.with_lock(|s| *s);
	// let b_lock_2 = b.with_lock(|s| *s);
	// assert_eq!(*a_lock_2 + *b_lock_2,5 );
	// }
	//! ```
	pub fn with_lock<F, U>(&self, func: F) -> U
	where
		F: FnOnce(&mut T) -> U,
	{
		let lock = self.data.lock();
		func(&mut *lock.unwrap())
	}

	// Construct the WithLock struct ith a Mutex.
	// ## Examples
	// ```rust
	// WithLock::<i64>::new(123);
	//```
	pub fn new<F>(data: Mutex<F>) -> WithLock<F> {
		WithLock { data }
	}
}
