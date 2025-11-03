pub mod syscallid;
pub mod fs;
pub mod process;

use crate::syscall::syscallid::SyscallID;
use crate::syscall::fs::sys_write;
use crate::syscall::process::sys_exit;
use crate::trap::fast::FastContext;

pub fn syscall(syscall_id: SyscallID, args: [usize; 3], ctx: &mut FastContext) -> isize {
	match syscall_id {
	    SyscallID::Write => sys_write(args[0], args[1] as *const u8, args[2]),
	    SyscallID::Exit => sys_exit(args[0] as i32),
	    SyscallID::GetTaskID => ctx.hart().cur_app as isize
	}
}