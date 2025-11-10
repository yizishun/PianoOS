
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit = 0,
    Ready = 1,
    Running = 2,
    Exited = 3,
}

impl From<TaskStatus> for u8 {
    #[inline] fn from(s: TaskStatus) -> u8 { s as u8 }
}

impl core::convert::TryFrom<u8> for TaskStatus {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, ()> {
        Ok(match v {
            0 => TaskStatus::UnInit,
            1 => TaskStatus::Ready,
            2 => TaskStatus::Running,
            3 => TaskStatus::Exited,
            _ => return Err(()),
        })
    }
}
