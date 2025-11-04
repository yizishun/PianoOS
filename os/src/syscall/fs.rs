use core::{slice::from_raw_parts, str::from_utf8};

use log::warn;

use crate::{config::APP_BASE_ADDR, global::{APP_MANAGER, USER_STACK}, harts::hart_context_in_trap_stage, print};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
	if !check_buf_valid(buf, len) {
		return -1;
	}
	match fd {
		FD_STDOUT => {
			let slice = unsafe { from_raw_parts(buf, len) };
			let str = from_utf8(slice).unwrap();
			print!("{}", str);
			len as isize
		}
		_ => {
			panic!("Unsupported fd in sys_write"); //TODO: kenel panic is not a good choice
		}
	}
}

fn check_buf_valid(buf: *const u8, len: usize) -> bool {
	let hart_context = hart_context_in_trap_stage();
	let harid = hart_context.hartid();
	let cur_app = hart_context.app_info.cur_app;
	let app_size = APP_MANAGER.get().unwrap().app_size(cur_app);

	let app_range = APP_BASE_ADDR as *const u8 .. ((APP_BASE_ADDR + app_size) as *const u8);
	let app_stack_range = unsafe { USER_STACK[harid].as_ptr_range() };
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
		warn!("app size: 0x{:x}", app_size);
		false
	}
}