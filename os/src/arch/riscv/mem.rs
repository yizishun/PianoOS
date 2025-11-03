use riscv::asm::fence_i;

use crate::arch::{common::ArchMem, riscv::Riscv64};
use log::error;
use core::arch::asm;

impl<C> ArchMem for Riscv64<C> {
	unsafe fn fencei(&self) {
		fence_i();
	}

	#[inline(never)]
	fn unwind(&self) {
		const MAX_DEPTH: usize = 128;
		let mut fp: usize;
		let mut prev_fp: usize;
		let mut ra: usize;
		let mut depth: usize = 0;
		let sp: usize;
		error!("==== Stack unwinding ====");
		unsafe {
			asm!(
				"mv {cur_sp}, sp",
				"mv {cur_fp}, fp",
				cur_sp = out(reg) sp,
				cur_fp = out(reg) fp,
				options(readonly)
			);
		}
		error!("[{:<10x}, {:<10x}]", sp, fp);
		while fp != 0 && depth < MAX_DEPTH {
			prev_fp = fp;
			unsafe {
				asm!(
					"ld {cur}, -16({prev})",
					"ld {ra}, -8({prev})",
					prev = in(reg) prev_fp,
					ra = out(reg) ra,
					cur = out(reg) fp,
					options(readonly)
				);
			}
			error!("[{:<10x}, {:<10x}] ra = {:<10x}", prev_fp, fp, ra);
			depth += 1;
		}
	}

}
