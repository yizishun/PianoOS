use core::{slice::from_raw_parts, str::from_utf8};

use crate::print;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
	match fd {
		FD_STDOUT => {
			let slice = unsafe {
				from_raw_parts(buf, len)
			};
			let str = from_utf8(slice).unwrap();
			print!("{}", str);
			len as isize
		}
		_ => {
			panic!("Unsupported fd in sys_write"); //TODO: kenel panic is not a good choice
		}
	}
}