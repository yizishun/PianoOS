pub mod hart;

use core::arch::asm;
use crate::arch::riscv;
use crate::arch::loongarch64;

// entry
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::entry;
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::entry;

// some common behavior
pub trait ArchISA {
    unsafe fn fencei();
}

pub trait ArchPower {
    fn shutdown(&self, fail: bool) -> !;
}

pub trait ArchTime {
    fn sleep(&self, sec: i32);
}

use crate::arch::riscv::*;
#[cfg(target_arch = "riscv64")]
type Arch = Riscv64<RiscvVirt>;//TODO: 这个RiscvVirt只是默认

// some common struct
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::FlowContext as FlowContext;
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::trap::FlowContext as FlowContext;