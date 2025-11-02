const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_GET_TASKID: usize = 1001;

#[repr(usize)]
pub enum SyscallID {
    	Write = SYSCALL_WRITE,
    	Exit = SYSCALL_EXIT,
	GetTaskID = SYSCALL_GET_TASKID
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyscallError {
	InvalidSyscallID
}

impl TryFrom<usize> for SyscallID {
	type Error = SyscallError;
	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			SYSCALL_WRITE => Ok(Self::Write),
			SYSCALL_EXIT => Ok(Self::Exit),
			SYSCALL_GET_TASKID => Ok(Self::GetTaskID),
			_ => Err(SyscallError::InvalidSyscallID)
		}
	}
}