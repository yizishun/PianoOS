pub mod syscallid;
pub mod fs;
pub mod process;

use crate::syscall::syscallid::SyscallID;
use crate::syscall::fs::sys_write;
use crate::syscall::process::{sys_exit, sys_get_taskid};

pub fn syscall(syscall_id: SyscallID, args: [usize; 3]) -> isize {
	match syscall_id {
	    SyscallID::Write => sys_write(args[0], args[1] as *const u8, args[2]),
	    SyscallID::Exit => sys_exit(args[0] as i32),
	    SyscallID::GetTaskID => sys_get_taskid()
	}
}