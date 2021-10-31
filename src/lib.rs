//! Sometimes, you are using a sync lock like the standard library's Mutex.
//! You need to use it in some async code, and call the `lock` method on the Mutex.
//! However, this is prone to deadlocks.
//! This crate provides a  wrapper around the Mutex that will never deadlock. (hence the description, Deadlock freedom)
//! It never deadlocks because it has its own function, `with_lock`, that access' the Mutex, locks it, performs
//! the closure, and then drops the lock.
//! Enjoy deadlock free code!

use std::sync::Mutex;

pub struct WithLock<T> {
	pub data: Mutex<T>,
}

impl<T> WithLock<T> {
	/// This function gives you access to what Mutex.lock().unwrap() would return.
	/// It will lock the Mutex, perform the closure, and then unlocks the Mutex.
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
	pub fn new<F>(data: Mutex<F>) -> WithLock<F> {
		WithLock { data }
	}
}

// This is a Mutex type for the people that really want 0 code changes.

pub struct MutexCell<T> {
	pub(crate) data: WithLock<T>,
}

impl<T> MutexCell<T> {
	/// Construct a new Mutex.
	/// ## What is going on
	/// This function creates a new [`Mutex`] where data (a field only visible in this crate) is a [`WithLock`]
	/// It then constructs the standard library's Mutex, which is then wrapped around by the [`WithLock`].
	/// The only change from using the standard library's Mutex is that all cases of `.lock().unwrap()` are handled for you,
	/// so you don't need to call .unwrap() after calling .lock().
	/// ## Example
	/// ```rust
	/// use with_lock::MutexCell;
	/// let mutex = MutexCell::new(23);
	/// assert_eq!(mutex.get(), 23)
	/// ```
	pub fn new(data: T) -> MutexCell<T> {
		MutexCell {
			data: WithLock::<T>::new(Mutex::new(data)),
		}
	}

	/// The get function. It gets the value inside the mutex.
	/// ## What is going on
	/// Locks the mutex and retrieves the value, then unlocks the mutex.
	pub fn get(&self) -> T
	where
		T: Copy,
	{
		self.data.with_lock(|s| *s)
	}

	/// The set function. It sets the value inside the mutex.
	/// ## What is going on
	/// Locks the mutex and updates the value, then unlocks the mutex.
	pub fn set(&self, data: T) {
		self.data.with_lock(|s| *s = data);
	}

	/// The replace function. It replaces the value inside the mutex and returns the previous value.
	/// ## What is going on
	/// Gets the value using the [`get`] method, then sets the value using the [`set`] method.
	pub fn replace(&self, new: T) -> T
	where
		T: Copy,
	{
		let old = self.get();
		self.set(new);
		old
	}
	/// The swap function. It swaps the value of one MutexCell with another.
	/// ## What is going on
	/// We get the current value using [`get`], then we call [`set`] on this cell and assign it to what [`get`] on the other cell returned. We then use [`set`] on the other cell to assign the value that [`get`] on this cell returned.
	pub fn swap(&self, other: &MutexCell<T>)
	where
		T: Copy,
	{
		let old = self.get();
		self.set(other.get());
		other.set(old);
	}
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

#[cfg(test)]
mod tests {
	use crate::*;

	#[test]
	fn test_mutex_cell_no_deadlocks() {
		let a = MutexCell::new(2);
		let b = MutexCell::new(3);
		let a_lock = a.get();
		let b_lock = b.get();
		assert_eq!(a_lock + b_lock, 5);
		let a_lock_2 = a.get();
		let b_lock_2 = b.get();
		assert_eq!(a_lock_2 + b_lock_2, 5);
	}

	#[test]
	fn test_mutex_cell_mutability() {
		let cell = MutexCell::new(3);
		assert_eq!(cell.get(), 3);
		cell.set(4);
		assert_eq!(cell.get(), 4);
	}

	#[test]
	fn test_mutex_cell_replace() {
		let cell = MutexCell::new(3);
		assert_eq!(cell.replace(4), 3);
		assert_eq!(cell.get(), 4);
	}

	#[test]
	fn test_mutex_cell_swap() {
		let c1 = MutexCell::new(5);
		let c2 = MutexCell::new(10);
		c1.swap(&c2);
		assert_eq!(10, c1.get());
		assert_eq!(5, c2.get());
	}
}
