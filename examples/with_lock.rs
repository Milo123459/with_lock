use std::sync::Mutex;
use with_lock::WithLock;

fn main() {
	let a = WithLock::<i32>::new(Mutex::new(1));
	let b = WithLock::<i32>::new(Mutex::new(2));
	println!("{:?}", a.with_lock(|x| *x) + b.with_lock(|x| *x));
}
