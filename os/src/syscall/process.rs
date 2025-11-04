use riscv::register::sepc;

use crate::global::APP_MANAGER;
use crate::harts::hart_context_in_trap_stage;
use crate::info;

pub fn sys_exit(xstate: i32) -> isize {
	let hart_context = hart_context_in_trap_stage();
	hart_context.print_syscall_record();
	info!("Application exited with code {}", xstate);
	APP_MANAGER.get().unwrap().run_next_app_in_trap();
	unsafe {
		sepc::write(crate::config::APP_BASE_ADDR);
	}
	0
}
