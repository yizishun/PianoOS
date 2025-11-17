use crate::syscall::syscallid::SyscallID;
use crate::arch::common::ArchTime;
use crate::TASK_MANAGER;
use crate::ARCH;
use strum::IntoEnumIterator;
use core::ops::Range;
use core::ptr::null;
use alloc::collections::BTreeMap;
use log::trace;
pub struct AppHartInfo {
	pub cur_app: usize,
	pub syscall_record: BTreeMap<SyscallID, usize>,
	pub app_range: Range<*const u8>,
	pub kernel_time: StopWatch,
	pub user_time: StopWatch,
}

impl AppHartInfo {
	pub fn new(app_id: usize, start_addr: usize, end_addr: usize) -> Self {
		let mut record = BTreeMap::new();
		for syscall in SyscallID::iter() {
			record.insert(syscall, 0);
		}
		AppHartInfo {
			cur_app: app_id, 
			syscall_record: record,
			app_range: start_addr as *const u8..end_addr as *const u8,
			kernel_time: StopWatch::new(),
			user_time: StopWatch::new(),
		}
	}

	pub fn end(&mut self) {
		self.print_app_statistics();
	}

	pub fn print_app_statistics(&self) {
		trace!("==== App({}) statistics ====", self.cur_app);
		trace!("Start addr: 0x{:x}", self.app_range.start as usize);
		trace!("End addr  : 0x{:x}", self.app_range.end as usize);
		trace!("Kernel total time: {}ns", self.kernel_time.time());
		trace!("User total time: {}ns", self.user_time.time());
		trace!("Syscall statistics --");
		self.print_syscall_record();
		trace!("== App({}) statistics end ==", self.cur_app);
	}

	pub fn print_syscall_record(&self) {
		for (syscall, count) in &self.syscall_record {
			trace!("{}: {}", syscall, count);
		}
	}

	pub fn clear_syscall_record(&mut self) {
		for syscall in SyscallID::iter() {
			*self.syscall_record.get_mut(&syscall).unwrap() = 0;
		}
	}
}

pub struct StopWatch {
	total_time: u64, //ns
	timing: bool,
	start_time: u64,
}

impl StopWatch {
	pub fn new() -> Self {
		Self { 
			total_time: 0,
			timing: false,
			start_time: 0,
		}
	}

	pub fn time(&self) -> u64 {
		self.total_time
	}

	pub fn start(&mut self) {
		assert_eq!(self.timing, false, "Stop Watch already start");
		self.timing = true;
		self.start_time = ARCH.time_ns();
	}

	pub fn end(&mut self) {
		assert_eq!(self.timing, true, "Stop Watch have not start");
		self.timing = false;
		let end_time = ARCH.time_ns();
		self.total_time += (end_time - self.start_time);
	}
}