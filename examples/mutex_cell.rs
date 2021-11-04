use with_lock::MutexCell;

fn main() {
	let a = MutexCell::new(2);
	let b = MutexCell::new(3);
	println!("{:?}", a.get() + b.get()); // 5
	b.set(4);
	println!("{:?}", a.get() + b.get()); // 6
	a.swap(&b);
	println!("A: {:?} B: {:?}", a.get(), b.get()); // A: 4 B: 2
	a.replace(4);
	println!("{:?}", a.get() + b.get()); // 8
}
