#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum ReadyLevel {
	Low,
	High	
}


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
	UnInit,
	Ready(ReadyLevel),
	Running,
	Exited,
}

impl From<TaskStatus> for u8 {
    	#[inline] fn from(s: TaskStatus) -> u8 { 
		match s {
		    TaskStatus::UnInit => 0,
		    TaskStatus::Running => 1,
		    TaskStatus::Exited => 2,
		    TaskStatus::Ready(ReadyLevel::Low) => 3,
		    TaskStatus::Ready(ReadyLevel::High) => 4
		}
	}
}

impl core::convert::TryFrom<u8> for TaskStatus {
	type Error = ();
	fn try_from(v: u8) -> Result<Self, ()> {
		Ok(match v {
			0 => TaskStatus::UnInit,
			1 => TaskStatus::Running,
			2 => TaskStatus::Exited,
			3 => TaskStatus::Ready(ReadyLevel::Low),
			4 => TaskStatus::Ready(ReadyLevel::High),
			_ => return Err(()),
		})
	}
}
