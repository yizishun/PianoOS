#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::TrapContext;
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::trap::TrapContext;