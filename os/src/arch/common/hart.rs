#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::hart::*;
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::hart::*;

pub const HART_INFO_SIZE: usize = size_of::<HartContext>();

#[repr(C)]
pub struct HartContext {
        #[cfg(target_arch = "riscv64")]
        flow_context: crate::arch::riscv::trap::FlowContext,
        #[cfg(target_arch = "loongarch64")]
        flow_context: crate::arch::loongarch64::trap::FlowContext,

        hartid: usize,
}

impl HartContext {
        pub fn get_hartnum() -> usize {
                crate::PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
        }

        pub fn get_hartid(&self) -> usize {
                self.hartid
        }
}

pub enum HartState {
        Started,
        Stoped,
        StartPeding,
        StopPeding,
        Invalid,
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
