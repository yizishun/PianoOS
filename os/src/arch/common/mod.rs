pub mod hart;

use crate::arch::riscv::*;

// entry
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::entry;
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::entry;

// some common behavior
pub trait ArchMem {
        unsafe fn fencei();
}

pub trait ArchPower {
        fn shutdown(&self, fail: bool) -> !;
}

pub trait ArchTime {
        fn sleep(&self, sec: i32);
}

#[cfg(target_arch = "riscv64")]
type Arch = Riscv64<RiscvVirt>; //TODO: 这个RiscvVirt只是默认
