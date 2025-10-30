use crate::arch::{common::ArchHarts, riscv::Riscv64};
use core::arch::asm;

impl<C> ArchHarts for Riscv64<C> {
	fn exchange_scratch(&self, mut val: usize) -> usize {
		unsafe { 
			asm!("csrrw {0}, sscratch, {0}", inlateout(reg) val) 
		};
    		val
	}

	fn get_scratch(&self) -> usize {
		let val;
		unsafe {
			asm!("csrr {0}, sscratch", out(reg) val)
		};
		val
	}
}