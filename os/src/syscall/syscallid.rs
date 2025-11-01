const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

#[repr(usize)]
pub enum SyscallID {
    	Write = SYSCALL_WRITE,
    	Exit = SYSCALL_EXIT
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
			_ => Err(SyscallError::InvalidSyscallID)
		}
	}
}