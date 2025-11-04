pub mod syscallid;
pub mod fs;
pub mod process;

use crate::harts::hart_context_in_trap_stage;
use crate::syscall::syscallid::SyscallID;
use crate::syscall::fs::sys_write;
use crate::syscall::process::sys_exit;

pub fn syscall(syscall_id: SyscallID, args: [usize; 3]) -> isize {
	let hart_context = hart_context_in_trap_stage();
	*hart_context.app_info.syscall_record.get_mut(&syscall_id).unwrap() += 1;
	match syscall_id {
	    	SyscallID::Write => {
			sys_write(args[0], args[1] as *const u8, args[2])
		},
		SyscallID::Exit => {
			sys_exit(args[0] as i32)
		},
		SyscallID::GetTaskID => {
			hart_context.app_info.cur_app as isize
		},
	}
}