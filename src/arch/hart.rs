#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::hart;

use crate::config::NUM_HART_MAX;

pub static mut HART_INFO: [HartInfo; NUM_HART_MAX] = [HartInfo::ZERO_HART; NUM_HART_MAX];
pub const HART_INFO_SIZE: usize = size_of::<HartInfo>();

#[derive(Clone, Copy)]
pub struct HartInfo {
    hartid: usize
}

impl HartInfo {
    pub const ZERO_HART: HartInfo = HartInfo { hartid: 0 };

    pub fn new(i: usize) -> Self {
        HartInfo { hartid: i }
    }

    pub fn get_hartnum() -> usize {
        crate::PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
    }

    pub fn get_cur_hart() -> &'static Self {
        #[cfg(target_arch = "riscv64")]
        hart::get_cur_hart()
    }
    pub fn get_hart_by_id(hartid: usize) -> &'static Self {
        unsafe { &HART_INFO[hartid] }
    }

    pub fn get_hartid(&self) -> usize {
        self.hartid
    }

    pub fn get_cur_hart_state(&self) -> HartState {
        #[cfg(target_arch = "riscv64")]
        hart::get_hart_state(self.hartid)
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
