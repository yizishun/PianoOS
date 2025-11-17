use crate::arch::common::ArchTime;
use crate::global::ARCH;
use crate::harts::task_context_in_trap_stage;
use crate::info;

pub fn sys_exit(xstate: i32) -> isize {
	info!("Application exited with code {}", xstate);
	0
}

pub fn sys_get_taskid() -> isize {
	task_context_in_trap_stage().app_info().cur_app as isize
}

pub fn sys_get_time() -> isize {
	ARCH.time_ms() as isize
}
