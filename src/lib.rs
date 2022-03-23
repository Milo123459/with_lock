//! A [`Mutex`](`std::sync::Mutex`) is prone to deadlocks when being used in sync and async code.
//! There are various causes of deadlocks, primarily due to the use of not dropping the lock.
//! This crate provides a function that locks for you, and automatically drops the lock. This eliminates the risk of deadlocks.
//!
//! With this crate, you would convert `.lock()` to simply be `.with_lock(|s| *s)`.
//!
//! # Features
//! - Simple API. Powered by parking_lot's [`Mutex`](`parking_lot::Mutex`).
//! - Provides a Cell like struct powered by a Mutex: [`MutexCell`](struct.MutexCell.html).
//!
//! # Caveats
//! These are the known causes of deadlocks. If you find a new one, please report it [here](https://github.com/Milo123459/with_lock/issues).
//!
//! `s.with_Lock(|test| s.with_lock(|test2| test2))`
//!
//! This code would deadlock. This is because we lock the Mutex then provide it as an argument, and because it the locked value isn't dropped it would deadlock.

use parking_lot::{const_mutex, Mutex};
use std::mem;
use std::ptr;

pub struct WithLock<T> {
	pub(crate) data: Mutex<T>,
}

impl<T> WithLock<T> {
	/// This function gives you access to what `Mutex.lock()` would return.
	///
	/// It will lock the Mutex, provide the locked value as an argument to the function, then drop the locked value.
	/// # Caveats
	/// If you clone the value inside the closure, everything touching that variable will need to be inside the closure.
	pub fn with_lock<F, U>(&self, function: F) -> U
	where
		F: FnOnce(&mut T) -> U,
	{
		let mut lock = self.data.lock();
		function(&mut *lock)
	}

	/// Construct the WithLock struct with a Mutex.
	/// ## Examples
	/// ```rust
	/// use with_lock::WithLock;
	/// WithLock::<i64>::new(123);
	/// ```
	pub fn new<F>(data: F) -> WithLock<F> {
		WithLock {
			data: const_mutex(data),
		}
	}
}

pub struct MutexCell<T> {
	pub(crate) data: WithLock<T>,
}

impl<T> MutexCell<T> {
	/// This function allows you to consturct a MutexCell with a value.
	/// ## Example
	/// ```rust
	/// use with_lock::MutexCell;
	/// let mutex = MutexCell::new(23);
	/// assert_eq!(mutex.get(), 23)
	/// ```
	pub fn new(data: T) -> MutexCell<T> {
		MutexCell {
			data: WithLock::<T>::new(data),
		}
	}

	/// This function gets the value inside the MutexCell.
	pub fn get(&self) -> T
	where
		T: Copy,
	{
		self.data.with_lock(|s| *s)
	}

	/// This function gets the value inside the MutexCell, but returns it as mutable.
	pub fn get_mut(&mut self) -> &mut T
	where
		T: Copy,
	{
		self.data.data.get_mut()
	}

	/// This function updates the value inside the Mutex.
	pub fn set(&self, data: T) {
		self.data.with_lock(|s| *s = data);
	}

	/// This function replaces the value inside the MutexCell with a new value.
	pub fn replace(&self, new: T) -> T {
		self.data.with_lock(|old| mem::replace(old, new))
	}

	/// This function swaps the value inside the MutexCell this is being called on (self) and on the one being provided as an argument.
	pub fn swap(&self, new: &MutexCell<T>) {
		if ptr::eq(self, new) {
			return;
		}
		self.data
			.with_lock(|a| new.data.with_lock(|b| mem::swap(a, b)))
	}

	/// This function replaces the value inside the MutexCell with [`Default::default`].
	pub fn take(&self) -> T
	where
		T: Default,
	{
		self.replace(Default::default())
	}

	/// This function takes the raw mutex and calls [`into_inner`](`std::sync::Mutex::into_inner`).
	pub fn into_inner(self) -> T {
		self.data.data.into_inner()
	}
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

#[cfg(test)]
mod tests {
	use crate::*;

	struct SharedData {
		pub a: i64,
		pub b: i64,
	}

	#[test]
	fn test_with_lock() {
		let a = WithLock::<i64>::new(2);
		let b = WithLock::<i64>::new(3);

		let action_and_get = |s: &mut i64| *s;
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
		let a = WithLock::<SharedData>::new(SharedData { a: 2, b: 2 });
		let b = WithLock::<SharedData>::new(SharedData { a: 3, b: 3 });

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
