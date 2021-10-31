//! * Sometimes, you are using a sync lock like the standard library's Mutex.
//! * You need to use it in some async code, and call the `lock` method on the Mutex.
//! * However, this is prone to deadlocks.
//! * This crate provides a safe wrapper around the Mutex that will never deadlock.
//! * It never deadlocks because it has its own function, [`with_lock`], that access' the Mutex, locks it, performs
//! * the closure, and then drops the lock.
//! * Enjoy deadlock free code!

use std::sync;

pub struct WithLock<T> {
	pub data: sync::Mutex<T>,
}

impl<T> WithLock<T> {
	/// This function gives you access to what Mutex.lock().unwrap() would return.
	/// It will lock the Mutex, perform the closure, and then unlock the Mutex.
	/// This function is safe to use in async code, but can also be used in sync code.
	/// ## Caveats
	/// If you clone the value inside the closure, everything touching that variable will need to be inside the closure.
	///
	/// ## Example
	/// ```rust
	/// use std::sync::Mutex;
	/// use with_lock::WithLock;
	///
	/// let a = WithLock::<i64>::new(Mutex::new(2));
	/// let b = WithLock::<i64>::new(Mutex::new(3));
	/// let a_lock = a.with_lock(|s| *s);
	/// let b_lock = b.with_lock(|s| *s);
	/// assert_eq!(a_lock + b_lock, 5);
	/// let a_lock_2 = a.with_lock(|s| *s);
	/// let b_lock_2 = b.with_lock(|s| *s);
	/// assert_eq!(a_lock_2 + b_lock_2,5 );
	///
	/// ```
	pub fn with_lock<F, U>(&self, func: F) -> U
	where
		F: FnOnce(&mut T) -> U,
	{
		let lock = self.data.lock();
		func(&mut *lock.unwrap())
	}

	/// Construct the WithLock struct with a Mutex.
	/// ## Examples
	/// ```rust
	/// use with_lock::WithLock;
	/// WithLock::<i64>::new(std::sync::Mutex::new(123));
	/// ```
	pub fn new<F>(data: sync::Mutex<F>) -> WithLock<F> {
		WithLock { data }
	}
}

// This is a Mutex type for the people that really want 0 code changes.

pub struct Mutex<T> {
	pub(crate) data: WithLock<T>,
}

impl<T: Copy> Mutex<T> {
	/// Construct a new Mutex.
	/// ## What is going on
	/// This function creates a new [`Mutex`] where data (a field only visible in this crate) is a [`WithLock`]
	/// It then constructs the standard library's Mutex, which is then wrapped around by the [`WithLock`].
	/// The only change from using the standard library's Mutex is that all cases of `.lock().unwrap()` are handled for you,
	/// so you don't need to call .unwrap() after calling .lock().
	/// ## Example
	/// ```rust
	/// use with_lock::Mutex;
	/// let mutex = Mutex::new(23);
	/// assert_eq!(mutex.lock(), 23)
	/// ```
	pub fn new(data: T) -> Mutex<T> {
		Mutex {
			data: WithLock::<T>::new(sync::Mutex::new(data)),
		}
	}

	/// The lock function. It does what it says.
	/// ## What is going on
	/// This function
	pub fn lock(&self) -> T {
		self.data.with_lock(|s| *s)
	}
}

#[cfg(test)]
mod tests {
	use crate::*;

	#[test]
	fn test_no_deadlocks_mutex() {
		let a = Mutex::new(2);
		let b = Mutex::new(3);
		let a_lock = a.lock();
		let b_lock = b.lock();
		assert_eq!(a_lock + b_lock, 5);
		let a_lock_2 = a.lock();
		let b_lock_2 = b.lock();
		assert_eq!(a_lock_2 + b_lock_2, 5);
	}
}
