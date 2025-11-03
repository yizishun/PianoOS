#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::get_taskid;

#[unsafe(no_mangle)]
fn main() -> i32 {
	let task_id = get_taskid();
	println!("task id {}", task_id);
	0
}
