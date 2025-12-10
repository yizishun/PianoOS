use crate::arch::riscv::*;
use crate::trap::fast::{ FastContext, FastResult };

// kernel entry
#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::entry;
#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::entry;

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
	fn time_ns(&self) -> u64;
	fn time_us(&self) -> u64;
	fn time_ms(&self) -> u64;
	fn time_s(&self) -> u64;
	fn set_next_timer_intr(&self, dur_ms: usize);
}

pub trait ArchHarts {
	fn exchange_scratch(&self, val: usize) -> usize;
	fn get_scratch(&self) -> usize;
	fn hart_start(&self, hartid: usize, start_addr: usize, opaque: usize); //TODO: may be error
}

pub trait ArchTrap {
	unsafe fn load_direct_trap_entry(&self);
	extern "C" fn fast_handler_user(
		ctx: FastContext,
		a1: usize,
		a2: usize,
		a3: usize,
		a4: usize,
		a5: usize,
		a6: usize,
		a7: usize,
	) -> FastResult;
	extern "C" fn fast_handler_kernel(
		ctx: FastContext,
		a1: usize,
		a2: usize,
		a3: usize,
		a4: usize,
		a5: usize,
		a6: usize,
		a7: usize,
	) -> FastResult;
	// app boot entry
	unsafe extern "C" fn boot_entry(a0: usize) -> !;
	// app boot prepare
	extern "C" fn boot_handler(start_addr: usize);
}

#[cfg(target_arch = "riscv64")]
pub type Arch = Riscv64<RiscvCommon>;

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::trap::FlowContext as FlowContext;