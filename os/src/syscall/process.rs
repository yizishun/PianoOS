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

//TODO: multi-hart will be a problem, because the id is acually hart local instead of global
pub fn sys_get_taskid() -> isize {
	let taskid_mux = APP_MANAGER.get().unwrap().next_app();
	let taskid = *taskid_mux;
	drop(taskid_mux);
	taskid as isize
}