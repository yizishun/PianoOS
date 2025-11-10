use crate::global::TASK_MANAGER;
use crate::harts::hart_context_in_trap_stage;
use crate::info;

pub fn sys_exit(xstate: i32) -> isize {
	hart_context_in_trap_stage().app_info.end();
	info!("Application exited with code {}", xstate);
	TASK_MANAGER.get().unwrap().run_next_at_trap();
	0
}
