use core::{slice::from_raw_parts, str::from_utf8};

use alloc::task;
use log::warn;

use crate::{global::{TASK_MANAGER, USER_STACK}, harts::{task_context_in_trap_stage}, print};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
	let addr_space = task_context_in_trap_stage()
		.addr_space();
	let phy_buf = addr_space
		.translated_byte_buffer(buf, len);
	// TODO: check
//	if !check_buf_valid(buf, len) {
//		return -1;
//	}
	match fd {
		FD_STDOUT => {
			for buf in phy_buf {
				let str = from_utf8(buf).unwrap();
				print!("{}", str);
			}
			len as isize
		}
		_ => {
			-1
		}
	}
}

fn check_buf_valid(buf: *const u8, len: usize) -> bool {
	let app_info = task_context_in_trap_stage().app_info();
	let cur_app = app_info.app_id;
	let app_size = TASK_MANAGER.get().unwrap().app_size(cur_app);

	let app_range = app_info.app_range.clone();
	let app_stack_range = unsafe { USER_STACK[cur_app].as_ptr_range() };
	if unsafe {
		(app_range.contains(&buf) && app_range.contains(&buf.add(len))) ||
		(app_stack_range.contains(&buf) && app_stack_range.contains(&buf.add(len)))
	} {
		true
	}
	else {
		warn!("buf out of scope");
		warn!("buf addr: 0x{:x}", buf as usize);
		warn!("buf size: 0x{:x}", len as usize);
		warn!("ustack start: 0x{:x}", app_stack_range.start as usize);
		warn!("ustack end  : 0x{:x}", app_stack_range.end as usize);
		warn!("app start: 0x{:x}", app_range.start as usize);
		warn!("app end:   0x{:x}", app_range.end as usize);
		warn!("app size: 0x{:x}", app_size);
		false
	}
}