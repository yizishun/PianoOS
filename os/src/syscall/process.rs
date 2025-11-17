use crate::harts::task_context_in_trap_stage;
use crate::info;

pub fn sys_exit(xstate: i32) -> isize {
	unsafe {
		(*task_context_in_trap_stage().app_info.get()).end();
	}
	info!("Application exited with code {}", xstate);
	0
}

pub fn sys_get_taskid() -> isize {
	unsafe {
		(*task_context_in_trap_stage().app_info.get()).cur_app as isize
	}
}
