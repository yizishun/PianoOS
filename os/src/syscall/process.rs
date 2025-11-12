use crate::global::TASK_MANAGER;
use crate::harts::task_context_in_trap_stage;
use crate::info;

pub fn sys_exit(xstate: i32) -> isize {
	unsafe {
		(*task_context_in_trap_stage().app_info.get()).end();
	}
	info!("Application exited with code {}", xstate);
	TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
	0
}

pub fn sys_get_taskid() -> isize {
	unsafe {
		(*task_context_in_trap_stage().app_info.get()).cur_app as isize
	}
}
