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
	pub start_time: usize,
	pub end_time: usize,
	pub app_range: Range<*const u8>
}

impl AppHartInfo {
	pub fn new() -> Self {
		let mut record = BTreeMap::new();
		for syscall in SyscallID::iter() {
			record.insert(syscall, 0);
		}
		AppHartInfo { 
			cur_app: 0, 
			syscall_record: record,
			start_time: 0,
			end_time: 0,
			app_range: null()..null()
		}
	}

	pub fn start(&mut self, cur_app: usize, app_range: Range<*const u8>) {
		self.cur_app = cur_app;
		self.clear_syscall_record();
		self.start_time = ARCH.time_ns();
		self.app_range = app_range;
	}

	pub fn end(&mut self) {
		self.end_time = ARCH.time_ns();
		self.print_app_statistics();
	}

	pub fn print_app_statistics(&self) {
		trace!("==== App({}) statistics ====", self.cur_app);
		trace!("Start addr: 0x{:x}", self.app_range.start as usize);
		trace!("End addr  : 0x{:x}", self.app_range.end as usize);
		trace!("Start time: {}ns", self.start_time);
		trace!("End time  : {}ns", self.end_time);
		trace!("Total time: {}ns", self.end_time - self.start_time);
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