use log::info;
use spin::{Mutex, MutexGuard};
use strum::IntoEnumIterator;

use crate::arch::common::{ArchMem, ArchTime};
use crate::global::{APP_MANAGER, ARCH};
use crate::arch::common::ArchPower;
use crate::config::MAX_APP_NUM;
use crate::global::_num_app;
use crate::harts::{hart_context_in_boot_stage, hart_context_in_trap_stage};
use crate::syscall::syscallid::SyscallID;
use alloc::collections::BTreeMap;
use log::trace;
pub struct AppManager {
	num_app: usize,
	next_app: Mutex<usize>,
	app_start_addr: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
	pub fn new() -> Self {
		let num_app_ptr: *const usize = core::ptr::addr_of!(_num_app);
		let num_app_usize: usize = unsafe { *num_app_ptr };
		let count: usize = num_app_usize + 1;
		let app_start_addr_raw: &[usize] =
			unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), count) };
		let mut app_start_addr: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
		app_start_addr[..count].copy_from_slice(app_start_addr_raw);
		AppManager { num_app: num_app_usize,
			     next_app: Mutex::new(0),
			     app_start_addr: app_start_addr }
	}

	pub fn print_app_info(&self) {
		info!("Kernel app number: {}", self.num_app);
		for i in 0..self.num_app {
			info!("app {i}: [{:<10p}, {:<10p}]",
			      self.app_start_addr[i] as *const usize,
			      self.app_start_addr[i + 1] as *const usize);
		}
	}

	pub fn next_app(&self) -> MutexGuard<usize> {
		self.next_app.lock()
	}

	pub fn load_app(&self, app_id: usize) {
		use crate::config::APP_BASE_ADDR;
		if app_id >= self.num_app {
			info!("All applications completed! Kennel shutdown");
			ARCH.shutdown(false);
		}
		let app_addr_start = *self.app_start_addr.get(app_id).unwrap();
		let app_addr_end = *self.app_start_addr.get(app_id + 1).unwrap();
		let count = app_addr_end - app_addr_start;
		let dst = APP_BASE_ADDR as *mut u8;
		info!("Kernel loading app({app_id})");
		unsafe {
			core::ptr::copy_nonoverlapping(app_addr_start as *const u8, dst, count);
			ARCH.fencei();
		}
	}

	pub fn run_next_app_in_boot(&self) {
		let mut next_app = self.next_app();
		self.load_app(*next_app);
		hart_context_in_boot_stage().app_info.start(*next_app);
		*next_app += 1;
	}

	pub fn run_next_app_in_trap(&self) {
		let mut next_app = self.next_app();
		self.load_app(*next_app);
		hart_context_in_trap_stage().app_info.start(*next_app);
		*next_app += 1;
	}
}

pub struct AppInfo {
	pub cur_app: usize,
	pub syscall_record: BTreeMap<SyscallID, usize>,
	pub start_time: usize,
	pub end_time: usize
}

impl AppInfo {
	pub fn new() -> Self {
		let mut record = BTreeMap::new();
		for syscall in SyscallID::iter() {
			record.insert(syscall, 0);
		}
		AppInfo { 
			cur_app: 0, 
			syscall_record: record,
			start_time: 0,
			end_time: 0
		}
	}

	pub fn start(&mut self, cur_app: usize) {
		self.cur_app = cur_app;
		self.clear_syscall_record();
		self.start_time = ARCH.time_ns();
	}

	pub fn end(&mut self) {
		self.end_time = ARCH.time_ns();
		self.print_app_statistics();
	}

	pub fn print_app_statistics(&self) {
		trace!("==== App statistics ====");
		trace!("Start time: {}ns", self.start_time);
		trace!("End time  : {}ns", self.end_time);
		trace!("Total time: {}ns", self.end_time - self.start_time);
		trace!("Syscall statistics --");
		self.print_syscall_record();
		trace!("== App statistics end ==");
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
