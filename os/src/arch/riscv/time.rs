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

	fn time_ns(&self) -> u64 {
		let time = time::read64();
		(time * 1_000_000_000) / VIRT_FREQUNCY as u64
	}

	fn time_us(&self) -> u64 {
		let time = time::read64();
		(time * 1_000_000) / VIRT_FREQUNCY as u64
	}

	fn time_ms(&self) -> u64 {
		let time = time::read64();
		(time * 1_000) / VIRT_FREQUNCY as u64
	}

	fn time_s(&self) -> u64 {
		let time = time::read64();
		time / VIRT_FREQUNCY as u64
	}

	fn set_next_timer_intr(&self, dur_ms: usize) {
		let now = time::read64();
		let total_clock_dur = (VIRT_FREQUNCY as u64 / 1_000) * dur_ms as u64;
		sbi_rt::set_timer(now + total_clock_dur);
	}
}
