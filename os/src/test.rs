use crate::{print, println};
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
	println!("\n{}============================================================{}", CYAN, RESET);
	println!("{}PianoOS Kernel Tests Support{}", BOLD, RESET);
	println!("{}Running {} tests on logic core{}", CYAN, tests.len(), RESET);
	println!("{}============================================================{}\n", CYAN, RESET);

	for test in tests {
		test.run();
	}

	println!("\n{}============================================================", GREEN);
	println!("SUCCESS: All {} tests passed!", tests.len());
	println!("============================================================{}{}", RESET, "\n");
}

pub trait Testable {
    	fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
	fn run(&self) {
		let symbol_name = core::any::type_name::<T>();
		println!("{}TEST: {}{}", CYAN, symbol_name, RESET);
		
		self();
		
		println!("{}RESULT: [ok]{}\n", GREEN, RESET);
	}
}

#[test_case]
fn trivial_assertion() {
	assert_eq!(1, 1);
}