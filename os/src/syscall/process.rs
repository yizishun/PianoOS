use riscv::register::sepc;

use crate::{global::APP_MANAGER, println};

pub fn sys_exit(xstate: i32) -> isize {
	println!("[kernel] Application exited with code {}", xstate);
	APP_MANAGER.get().unwrap().run_next_app();
	unsafe {
		sepc::write(crate::config::APP_BASE_ADDR);
	}
	0
}