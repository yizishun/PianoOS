use crate::arch::riscv::Riscv64;
use crate::arch::{common::ArchTime, riscv::RiscvVirt};
use crate::config::VIRT_FREQUNCY;
use riscv::register::{sie, time};

impl ArchTime for Riscv64<RiscvVirt> {
	fn sleep(&self, sec: i32) {
		let time_start = time::read();
		let time_end = time_start + (VIRT_FREQUNCY * 100_0000 * sec) as usize;
		unsafe {
			sie::set_stimer();
		}
		sbi_rt::set_timer(time_end as u64);
		riscv::asm::wfi();
	}

	fn time_ns(&self) -> usize {
	    	let time = time::read64();
		((1000u64 / VIRT_FREQUNCY as u64) * time) as usize
	}
}
