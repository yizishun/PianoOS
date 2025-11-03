#![macro_use]
#![allow(static_mut_refs)]
use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{self, Write};
use spin::Mutex;

use crate::global::PLATFORM;

#[derive(Clone, Copy, Debug)]
pub enum ConsoleType {
	Uart16550U8,
	Uart16550U32,
	RiscvSbi,
}

impl ConsoleType {
	pub fn compatible(device: &str) -> Option<ConsoleType> {
		use ConsoleType::*;
		if ["ns16550a"].contains(&device) {
			Some(Uart16550U8)
		} else if ["snps,dw-apb-uart"].contains(&device) {
			Some(Uart16550U32)
		} else {
			None
		}
	}
}

/// console device driver should impl this trait
pub(crate) trait ConsoleDevice: Send {
	/// read bytes from console input
	fn read(&self, buf: &mut [u8]) -> usize;
	/// write bytes to console output
	fn write(&self, buf: &[u8]) -> usize;
}

pub struct KernelConsole {
	inner: Mutex<Box<dyn ConsoleDevice>>, //spinlock
}

impl KernelConsole {
	pub fn new(inner: Mutex<Box<dyn ConsoleDevice>>) -> Self {
		Self { inner }
	}
}

impl fmt::Write for &KernelConsole {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		let console = self.inner.lock();
		let mut bytes = s.as_bytes();
		while !bytes.is_empty() {
			let count = console.write(bytes);
			bytes = &bytes[count..];
		}
		Ok(()) //TODO: error handle
	}
}

pub fn print(args: fmt::Arguments) {
	let mut s = String::new();
	let _ = core::fmt::write(&mut s, args); //TODO: error handle
	PLATFORM.get()
		.unwrap()
		.board_device
		.console
		.as_ref()
		.unwrap()
		.write_str(&s)
		.unwrap();
}
