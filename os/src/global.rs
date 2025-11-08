use crate::task::TaskManager;
use crate::config::{MAX_APP_NUM, NUM_HART_MAX};
use crate::loader::{ElfInfo, LoaderElfInfo};
use crate::mm::stack::{KernelStack, UserStack};
use crate::platform::Platform;
use crate::arch::common::Arch;
use spin::Once;

unsafe extern "C" {
	pub static skernel: usize;
	pub static stext: usize;
	pub static etext: usize;
	pub static srodata: usize;
	pub static erodata: usize;
	pub static sdata: usize;
	pub static edata: usize;
	pub static skstack: usize;
	pub static ekstack: usize;
	pub static sustack: usize;
	pub static eustack: usize;
	pub static sheap: usize;
	pub static eheap: usize;
	pub static sbss: usize;
	pub static ebss: usize;
	pub static ekernel: usize;

	pub static _num_app: usize;
}

pub static PLATFORM: Once<Platform> = Once::new();

pub static TASK_MANAGER: Once<TaskManager> = Once::new();

pub static LOADER_ELF_INFO: Once<LoaderElfInfo> = Once::new();

#[unsafe(link_section = ".bss.kstack")]
pub static mut KERNEL_STACK: [KernelStack; NUM_HART_MAX] = [KernelStack::ZERO; NUM_HART_MAX];

#[unsafe(link_section = ".bss.ustack")]
pub static mut USER_STACK: [UserStack; NUM_HART_MAX] = [UserStack::ZERO; NUM_HART_MAX];

pub static ARCH: Arch = Arch::new();