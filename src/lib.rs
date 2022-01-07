//! A `Mutex` is prone to deadlocks when being used in sync and async code.
//! It can lock easily, having a task do something to the value, ie, locking it, whilst having another task do the same thing.
//! This is the most common cause of something called a deadlock.
//!
//! There are many takes on preventing deadlocks, but this is the simplest and easiest to migrate to, due to it having a very similar API.
//!
//! For instance, converting something like `.lock().unwrap()...` to use `with_lock`'s API would look something like `.with_lock(|s| s...)`.
//! This is an example of how easy to migrate something to use `with_lock` is.
//!
//! # Caveats
//! This code would deadlock:
//! `s.with_Lock(|test| s.with_lock(|test2| test2))`
//!
//! This is because the internal code just locks the Mutex and provides it as an argument then dropping it once the function has finished executing. This is locking whilst the function is still executing, hence the deadlock.
//!
//! # Features
//! - This crate uses no unsafe code directly.
//! - Provides a Cell like struct powered by a Mutex: [`MutexCell`](struct.MutexCell.html)
//! - No dependencies

use std::mem;
use std::ptr;
use std::sync::Mutex;

pub struct WithLock<T> {
	pub(crate) data: Mutex<T>,
}

impl<T> WithLock<T> {
	/// This function gives you access to what `Mutex.lock().unwrap()` would return.
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
	/// let action_and_get = |s: &mut i32| *s;
	/// let a_lock = a.with_lock(action_and_get);
	/// let b_lock = b.with_lock(action_and_get);

	/// assert_eq!(a_lock + b_lock, 5);
	/// let a_lock_2 = a.with_lock(|s| *s);
	/// let b_lock_2 = b.with_lock(|s| *s);
	/// assert_eq!(a_lock_2 + b_lock_2, 5);
	///
	/// ```
	pub fn with_lock<F, U>(&self, function: F) -> U
	where
		F: FnOnce(&mut T) -> U,
	{
		let lock = self.data.lock();
		function(&mut *lock.unwrap())
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

	/// The get_mut function. It gets the value inside the mutex and returns it as mutable.
	/// ## What is going on
	/// Locks the mutex and retrieves the value, then unlocks the mutex.
	pub fn get_mut(&mut self) -> &mut T
	where
		T: Copy,
	{
		self.data.data.get_mut().unwrap()
	}

	/// The set function. It sets the value inside the mutex.
	/// ## What is going on
	/// Locks the mutex and updates the value, then unlocks the mutex.
	pub fn set(&self, data: T) {
		self.data.with_lock(|s| *s = data);
	}

	/// The replace function. It replaces the value inside the mutex and returns the previous value.
	/// ## What is going on
	/// Locks this cell, and then calls `mem::replace` on the locked value.
	pub fn replace(&self, new: T) -> T {
		self.data.with_lock(|old| mem::replace(old, new))
	}
	/// The swap function. It swaps the value of one MutexCell with another.
	/// ## What is going on
	/// Locks this cell, then locks `new`. Then, we swap the data using `mem::swap`.
	pub fn swap(&self, new: &MutexCell<T>) {
		if ptr::eq(self, new) {
			return;
		}
		self.data
			.with_lock(|a| new.data.with_lock(|b| mem::swap(a, b)))
	}

	/// The take function. It takes the value from the Mutex, returns it and sets the value to `Default::default()`
	/// ## What is going on
	/// Calls the `replace` function, and then sets it to `Default::default()`.

	pub fn take(&self) -> T
	where
		T: Default,
	{
		self.replace(Default::default())
	}

	/// The into_inner function. It takes the Mutex and calls `into_inner` on it.
	/// ## What is going on
	/// It takes the Mutex and calls `into_inner` on it.

	pub fn into_inner(self) -> T {
		self.data.data.into_inner().unwrap()
	}
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

#[cfg(test)]
mod tests {
	use crate::*;

	struct SharedData {
		pub a: i32,
		pub b: i32,
	}

	#[test]
	fn test_with_lock() {
		let a = WithLock::<i64>::new(Mutex::new(2));
		let b = WithLock::<i64>::new(Mutex::new(3));

		let action_and_get = |s: &mut i32| *s;
		let a_lock = a.with_lock(action_and_get);
		let b_lock = b.with_lock(action_and_get);
		assert_eq!(a_lock + b_lock, 5);

		// repeat action with embedded lambda expression
		let a_lock_2 = a.with_lock(|s| *s);
		let b_lock_2 = b.with_lock(|s| *s);
		assert_eq!(a_lock_2 + b_lock_2, 5);
	}

	#[test]
	fn test_with_lock_over_struct() {
		let a = WithLock::<SharedData>::new(Mutex::new(SharedData { a: 2, b: 2 }));
		let b = WithLock::<SharedData>::new(Mutex::new(SharedData { a: 3, b: 3 }));

		let action_and_get = |s: &mut SharedData| (*s).a;
		let a_lock = a.with_lock(action_and_get);
		let b_lock = b.with_lock(action_and_get);
		assert_eq!(a_lock + b_lock, 5);

		// repeat action with embedded lambda expression and member b (avoid dead code warning)
		let a_lock_2 = a.with_lock(|s| (*s).b);
		let b_lock_2 = b.with_lock(|s| (*s).b);
		assert_eq!(a_lock_2 + b_lock_2, 5);
	}

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

	#[test]
	fn test_mutex_cell_swap_doesnt_deadlock() {
		let c1 = MutexCell::new(5);
		assert_eq!(c1.get(), 5);
		c1.swap(&c1);
		assert_eq!(c1.get(), 5);
	}

	#[test]
	fn test_mutex_cell_get_mut() {
		let mut c1 = MutexCell::new(5);
		assert_eq!(c1.get(), 5);
		let c2 = c1.get_mut();
		*c2 += 1;
		assert_eq!(c1.get(), 6);
	}

	#[test]
	fn test_mutex_cell_take() {
		let c = MutexCell::new(5);
		let five = c.take();

		assert_eq!(five, 5);
		assert_eq!(c.into_inner(), 0);
	}

	#[test]
	fn test_mutex_cell_into_inner() {
		let c = MutexCell::new(5);
		let five = c.into_inner();

		assert_eq!(five, 5);
	}
}
