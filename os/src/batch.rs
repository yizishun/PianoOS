use core::ops::Range;
use core::ptr::null;

use log::info;
use riscv::register::mstatus::set_fs;
use spin::{Mutex, MutexGuard};
use strum::IntoEnumIterator;

use crate::arch::common::{ArchMem, ArchTime};
use crate::global::{APP_MANAGER, ARCH};
use crate::arch::common::ArchPower;
use crate::config::MAX_APP_NUM;
use crate::global::_num_app;
use crate::harts::{hart_context_in_boot_stage, hart_context_in_trap_stage};
use crate::syscall::syscallid::SyscallID;
use crate::elf::ElfInfo;
use alloc::collections::BTreeMap;
use log::trace;
use riscv::register::sepc;
pub struct AppManager {
	num_app: usize,
	next_app: Mutex<usize>,
	elf_info: [ElfInfo; MAX_APP_NUM]
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
		let mut elf_info: [ElfInfo; MAX_APP_NUM] = [ElfInfo::ZERO; MAX_APP_NUM];
		for (elf, i) in elf_info.iter_mut().zip(0..num_app_usize) {
			elf.start_addr = app_start_addr[i];
			elf.end_addr = app_start_addr[i + 1];
		}
		AppManager { num_app: num_app_usize,
			     next_app: Mutex::new(0),
			     elf_info: elf_info }
	}

	pub fn elf_info(&self, idx: usize) -> &ElfInfo {
		self.elf_info.get(idx).unwrap()
	}

	pub fn print_app_info(&self) {
		info!("Kernel app number: {}", self.num_app);
		for i in 0..self.num_app {
			info!("app {i}: [{:<10p}, {:<10p}]",
			      self.elf_info[i].start_addr as *const usize,
			      self.elf_info[i].end_addr as *const usize);
		}
	}

	pub fn app_size(&self, app_id: usize) -> usize {
		assert!(app_id < self.num_app, "Invalid app id {}", app_id);
		let size: isize = (self.elf_info[app_id].end_addr - self.elf_info[app_id].start_addr) as isize;
		assert!(size >= 0, "app size is nagative");
		size as usize
	}

	pub fn next_app(&self) -> MutexGuard<usize> {
		self.next_app.lock()
	}

	pub fn load_app_elf(&self, app_id: usize) -> Range<*const u8>{
		if app_id >= self.num_app {
			info!("All applications completed! Kennel shutdown"); //TODO:这个打印要是发生在boot time就会出错
			ARCH.shutdown(false);
		}
		self.elf_info.get(app_id).unwrap().load_elf()
	}

	pub fn run_next_app_in_boot(&self) {
		let mut next_app = self.next_app();
		let app_range = self.load_app_elf(*next_app);
		hart_context_in_boot_stage().app_info.start(*next_app, app_range.clone());
		*next_app += 1;
		unsafe {
			sepc::write(app_range.start as usize);
		}
	}

	pub fn run_next_app_in_trap(&self) {
		let mut next_app = self.next_app();
		let app_range = self.load_app_elf(*next_app);
		info!("Kernel loading app({})", *next_app);
		hart_context_in_trap_stage().app_info.start(*next_app, app_range.clone());
		*next_app += 1;
		unsafe {
			sepc::write(app_range.start as usize);
		}
	}
}

pub struct AppInfo {
	pub cur_app: usize,
	pub syscall_record: BTreeMap<SyscallID, usize>,
	pub start_time: usize,
	pub end_time: usize,
	pub app_range: Range<*const u8>
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
		trace!("==== App statistics ====");
		trace!("Start addr: 0x{:x}", self.app_range.start as usize);
		trace!("End addr  : 0x{:x}", self.app_range.end as usize);
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
