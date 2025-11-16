use crate::arch::riscv::Riscv64;
use crate::arch::{common::ArchTime, riscv::RiscvVirt};
use crate::config::VIRT_FREQUNCY;
use riscv::register::{sie, time};

impl ArchTime for Riscv64<RiscvVirt> {
	fn sleep(&self, sec: usize) {
		let time_start = time::read();
		let time_end = time_start + (VIRT_FREQUNCY * sec) as usize;
		unsafe {
			sie::set_stimer();
		}
		sbi_rt::set_timer(time_end as u64);
		riscv::asm::wfi();
	}

	fn enable_timer(&self) {
		unsafe {
			sie::set_stimer();
		}
	}

	fn time_ns(&self) -> usize {
		let time = time::read64();
		(time as usize * 1_000_000_000) / VIRT_FREQUNCY
	}

	fn time_us(&self) -> usize {
		let time = time::read64();
		(time as usize * 1_000_000) / VIRT_FREQUNCY
	}

	fn time_ms(&self) -> usize {
		let time = time::read64();
		(time as usize * 1_000) / VIRT_FREQUNCY
	}

	fn time_s(&self) -> usize {
		let time = time::read64();
		time as usize / VIRT_FREQUNCY
	}

	fn set_next_timer_intr(&self, dur_ms: usize) {
		let now = time::read64();
		let total_clock_dur = (VIRT_FREQUNCY as u64 / 1_000) * dur_ms as u64;
		sbi_rt::set_timer(now + total_clock_dur);
	}
}
