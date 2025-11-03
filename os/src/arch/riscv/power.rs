use crate::arch::{common::ArchPower, riscv::Riscv64};

impl<C> ArchPower for Riscv64<C> {
	fn shutdown(&self, fail: bool) -> ! {
		use sbi_rt::{NoReason, Shutdown, SystemFailure};
		if fail {
			sbi_rt::system_reset(Shutdown, SystemFailure);
		} else {
			sbi_rt::system_reset(Shutdown, NoReason);
		}
		loop {}
	}
}
