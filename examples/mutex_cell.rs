use with_lock::MutexCell;

fn main() {
	let mut a = MutexCell::new(2);
	let b = MutexCell::new(3);
	println!("{}", a.get() + b.get()); // 5
	b.set(4);
	println!("{}", a.get() + b.get()); // 6
	a.swap(&b);
	println!("A: {} B: {}", a.get(), b.get()); // A: 4 B: 2
	let old = a.replace(5);
	println!("old: {}", old); // 4
	let old_2 = a.replace(4);
	println!("old 2: {}", old_2); // 5
	println!("{}", a.get() + b.get()); // 8
	let cell = a.get_mut();
	*cell += 1;
	println!("{}", a.get()); // 5

	let a_new = a.take();
	println!("a: {}", a.into_inner());
	println!("a new: {}", a_new);
}
