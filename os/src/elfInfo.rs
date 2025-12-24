use core::{array};
use crate::{config::MAX_APP_NUM};
use crate::_num_app;
use crate::info;
pub struct ElfsInfo {
	pub num_app: usize,
	elf_info: [&'static [u8]; MAX_APP_NUM]
}

impl ElfsInfo {
	pub fn new() -> Self {
		let num_app_ptr: *const usize = core::ptr::addr_of!(_num_app);
		let num_app_usize: usize = unsafe { *num_app_ptr };
		let count: usize = num_app_usize + 1;
		let app_start_addr_raw: &[usize] =
			unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), count) };
		let mut app_start_addr: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
		app_start_addr[..count].copy_from_slice(app_start_addr_raw);
		let elf_info: [&[u8]; MAX_APP_NUM] = array::from_fn(|i| {
			if i >= num_app_usize {
				return &[] as &[u8];
			}
			let len = app_start_addr[i+1] - app_start_addr[i];
			unsafe {
				core::slice::from_raw_parts(app_start_addr[i] as *const u8, len)
			}
		});
		Self {
			num_app: num_app_usize,
			elf_info
		}
	}

	pub fn elf_info(&self, idx: usize) -> &[u8] {
		self.elf_info.get(idx).unwrap()
	}

	pub fn print_app_info(&self) {
		info!("Kernel app number: {}", self.num_app);
	}

	pub fn app_size(&self, app_id: usize) -> usize {
		assert!(app_id < self.num_app, "Invalid app id {}", app_id);
		self.elf_info[app_id].len()
	}

}
