use crate::arch::riscv::*;

// kernel entry
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::entry;
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::entry;

// app boot enrty
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::boot_entry;
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::boot_handler;

// fast handler
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::handler::fast_handler;

// some common behavior
pub trait ArchMem {
	unsafe fn fencei(&self);
	fn unwind(&self);
}

pub trait ArchPower {
	fn shutdown(&self, fail: bool) -> !;
}

pub trait ArchTime {
	fn sleep(&self, sec: usize);
	fn enable_timer(&self);
	fn time_ns(&self) -> usize;
	fn time_us(&self) -> usize;
	fn time_ms(&self) -> usize;
	fn time_s(&self) -> usize;
	fn set_next_timer_intr(&self, dur_ms: usize);
}

pub trait ArchHarts {
	fn exchange_scratch(&self, val: usize) -> usize;
	fn get_scratch(&self) -> usize;
}

pub trait ArchTrap {
	unsafe fn load_direct_trap_entry(&self);
	unsafe fn set_next_pc(&self, addr: usize);
	unsafe fn set_next_user_stack(&self, addr: usize);
}

#[cfg(target_arch = "riscv64")]
pub type Arch = Riscv64<RiscvVirt>; //TODO: 这个RiscvVirt只是默认

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::FlowContext as FlowContext;