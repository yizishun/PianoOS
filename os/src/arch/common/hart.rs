#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::hart::*;
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::hart::*;

pub const HART_INFO_SIZE: usize = size_of::<HartContext>();

#[repr(C)]
pub struct HartContext {
    trap_context: super::FlowContext,

    hartid: usize
}

impl HartContext {
    pub fn get_hartnum() -> usize {
        crate::PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
    }

    pub fn get_hartid(&self) -> usize {
        self.hartid
    }

    pub fn get_cur_hart_state(&self) -> HartState {
        #[cfg(target_arch = "riscv64")]
        { super::riscv::hart::get_hart_state(self.hartid) }
        #[cfg(target_arch = "loongarch64")]
        { super::riscv::hart::get_hart_state(self.hartid) }
    }
}

pub enum HartState {
    Started,
    Stoped,
    StartPeding,
    StopPeding,
    Invalid
}

impl core::fmt::Display for HartState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HartState::Started => write!(f, "Hart is Started"),
            HartState::Stoped => write!(f, "Hart is Stoped"),
            HartState::StartPeding => write!(f, "Hart is Start pending"),
            HartState::StopPeding => write!(f, "Hart is Stop pending"),
            HartState::Invalid => write!(f, "Hart is Invalid"),
        }
    }
}
