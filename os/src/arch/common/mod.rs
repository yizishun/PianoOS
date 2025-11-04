use crate::arch::riscv::*;

// kernel entry
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::entry;
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::entry;

// app boot enrty
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::boot_entry;

// fast handler
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::fast_handler;

// some common behavior
pub trait ArchMem {
	unsafe fn fencei(&self);
	fn unwind(&self);
}

pub trait ArchPower {
	fn shutdown(&self, fail: bool) -> !;
}

pub trait ArchTime {
	fn sleep(&self, sec: i32);
	fn time_ns(&self) -> usize;
}

pub trait ArchHarts {
	fn exchange_scratch(&self, val: usize) -> usize;
	fn get_scratch(&self) -> usize;
}

pub trait ArchTrap {
	unsafe fn load_direct_trap_entry(&self);
}

#[cfg(target_arch = "riscv64")]
pub type Arch = Riscv64<RiscvVirt>; //TODO: 这个RiscvVirt只是默认

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::FlowContext as FlowContext;