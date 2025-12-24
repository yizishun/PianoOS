extern crate alloc;
pub const HART_INFO_SIZE: usize = size_of::<HartContext>();

use core::ptr::NonNull;
use core::arch::asm;

// Make sure HartContext is aligned.
//
// HartContext will always at the end of Stack, so we should make sure
// STACK_SIZE_PER_HART is a multiple of b.
use crate::{arch::common::{ArchHarts, FlowContext}, config::KERNEL_STACK_SIZE, global::ARCH, task::{block::TaskControlBlock, harts::AppHartInfo}, trap::{LoadedTrapStack, TrapHandler}};
const _: () = assert!(KERNEL_STACK_SIZE % core::mem::align_of::<HartContext>() == 0);

#[repr(C, align(128))]
pub struct HartContext {
	hartid: usize,
	// for switch addr space
	kaddr_space: usize,
	ksp: usize,
}

impl HartContext {
	pub fn init(&mut self, hartid: usize){
		self.hartid = hartid;
	}

	pub fn get_hartnum() -> usize {
		crate::PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
	}

	pub fn hartid(&self) -> usize {
		self.hartid
	}
}

pub fn trap_handler_in_trap_stage() -> &'static mut TrapHandler {
	let mut scratch: *mut TrapHandler;
	unsafe {
		asm!("mv {}, tp", out(reg) scratch);
		scratch.as_mut().unwrap()
	}
}

pub fn hart_context_in_boot_stage() -> &'static mut HartContext {
	let scratch = ARCH.get_scratch() as *mut TrapHandler;
	let mut hart_context = unsafe { (*scratch).hart };
	unsafe {
		hart_context.as_mut()
	}
}

pub fn hart_context_in_trap_stage() -> &'static mut HartContext {
	let mut scratch: *mut TrapHandler;
	unsafe {
		asm!("mv {}, tp", out(reg) scratch);
	}
	let mut hart_context = unsafe { (*scratch).hart };
	unsafe {
		hart_context.as_mut()
	}
}

pub fn task_block_in_boot_stage() -> &'static mut TaskControlBlock {
	let scratch = ARCH.get_scratch() as *mut TrapHandler;
	let task_block = unsafe { (*scratch).context.as_ptr() as *mut TaskControlBlock };
	unsafe {
		task_block.as_mut().unwrap()
	}
}

pub fn task_context_in_trap_stage() -> &'static mut TaskControlBlock {
	let mut scratch: *mut TrapHandler;
	unsafe {
		asm!("mv {}, tp", out(reg) scratch);
	}
	let task_block = unsafe { (*scratch).context.as_ptr() as *mut TaskControlBlock };
	unsafe {
		task_block.as_mut().unwrap()
	}
}

pub enum HartState {
	Started,
	Stoped,
	StartPeding,
	StopPeding,
	Invalid,
}

impl core::fmt::Display for HartState {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			HartState::Started => write!(f, "Hart is Started"),
			HartState::Stoped => write!(f, "Hart is Stoped"),
			HartState::StartPeding => write!(f, "Hart is Start pending"),
			HartState::StopPeding => write!(f, "Hart is Stop pending"),
			HartState::Invalid => write!(f, "Hart is Invalid"),
		}
	}
}