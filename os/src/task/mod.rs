use core::ops::Range;
use core::ptr::null;
use core::sync::atomic::{AtomicBool, Ordering};
use core::array;

use log::info;
use spin::{Mutex, MutexGuard};

use crate::global::{ARCH, LOADER_ELF_INFO};
use crate::arch::common::ArchPower;
use crate::config::MAX_APP_NUM;
use crate::global::_num_app;
use crate::harts::{hart_context_in_boot_stage, hart_context_in_trap_stage};
use riscv::register::sepc;

pub mod harts;

pub struct TaskManager {
	pub num_app: usize,
	next_app: Mutex<usize>,
	finished: [AtomicBool; MAX_APP_NUM],
	app_range: [Range<*const u8>; MAX_APP_NUM]
}

unsafe impl Send for TaskManager {}
unsafe impl Sync for TaskManager {}

impl TaskManager {
	pub fn new() -> Self {
		let num_app_ptr: *const usize = core::ptr::addr_of!(_num_app);
		let num_app_usize: usize = unsafe { *num_app_ptr };
		let finished: [AtomicBool; MAX_APP_NUM] =
       			array::from_fn(|_| AtomicBool::new(false));
		let app_range: [Range<*const u8>; MAX_APP_NUM] = 
			array::from_fn(|i| {
				if i < num_app_usize {
					LOADER_ELF_INFO.get().unwrap()
						.load(i)
				} else {
					Range { start: null(), end:  null()}
				}
			});
		TaskManager { num_app: num_app_usize,
			     next_app: Mutex::new(0),
			     finished,
			     app_range
			}
	}

	pub fn next_app(&self) -> MutexGuard<usize> {
		self.next_app.lock()
	}

	pub fn app_size(&self, app_id: usize) -> usize {
		let app_range = self.app_range.get(app_id).unwrap();
		let start = app_range.start as usize;
		let end = app_range.end as usize;
		end - start
	}

	pub fn check_end(&self, app_id: usize) {
		if app_id >= self.num_app {
			loop {
			    	let all_finished = 
			    		self.finished.iter().take(self.num_app)
						.all(|f| f.load(Ordering::Acquire));
				if all_finished {
					info!("All applications completed! Kennel shutdown");
					ARCH.shutdown(false);
				} else {
					//info!("Waiting other program finished");
					//self.finished.iter().take(self.num_app).enumerate().for_each(|e| {
						//info!("program({:?}) finished: {:?}", e.0, e.1);
					//});
				}
			}
		}
	}

	pub fn run_next_app_in_boot(&self) {
		let mut next_app = self.next_app();
		self.check_end(*next_app);
		let app_range = &self.app_range[*next_app];
		hart_context_in_boot_stage().app_info.start(*next_app, app_range.clone());
		*next_app += 1;
		unsafe {
			sepc::write(app_range.start as usize);
		}
	}

	pub fn run_next_app_in_trap(&self) {
		let mut next_app = self.next_app();
		self.check_end(*next_app);
		let app_range = &self.app_range[*next_app];
		info!("Kernel loading app({})", *next_app);
		hart_context_in_trap_stage().app_info.start(*next_app, app_range.clone());
		*next_app += 1;
		unsafe {
			sepc::write(app_range.start as usize);
		}
	}

	pub fn set_finish(&self, app_id: usize) {
		self.finished.get(app_id).unwrap()
			.store(true, Ordering::Release);
	}
}
