use strum_macros::EnumIter;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TASKID: usize = 1001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
#[repr(usize)]
pub enum SyscallID {
    	Write = SYSCALL_WRITE,
    	Exit = SYSCALL_EXIT,
	GetTaskID = SYSCALL_GET_TASKID,
	Yield = SYSCALL_YIELD,
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
			SYSCALL_YIELD => Ok(Self::Yield),
			_ => Err(SyscallError::InvalidSyscallID)
		}
	}
}

impl core::fmt::Display for SyscallID {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Exit => write!(f, "Exit"),
			Self::GetTaskID => write!(f, "GetTaskID"),
			Self::Write => write!(f, "Write"),
			Self::Yield => write!(f, "Yield"),
		}
	}
}