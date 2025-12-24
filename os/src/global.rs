use crate::mm::addr_space::AddrSpace;
use crate::mm::frame_allocator::{FrameAllocator, StackFrameAllocator};
use crate::task::TaskManager;
use crate::config::{MAX_APP_NUM, NUM_HART_MAX};
use crate::elfInfo::ElfsInfo;
use crate::mm::stack::{KernelStack, UserStack};
use crate::platform::Platform;
use crate::arch::common::Arch;
use spin::Once;

unsafe extern "C" {
	// in kernel linker script
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
	pub static sbss_nostack: usize;
	pub static ebss: usize;
	pub static ekernel: usize;
	pub static strampoline: usize;

	// in app link asm
	pub static _num_app: usize;
}

pub static PLATFORM: Once<Platform> = Once::new();

pub static TASK_MANAGER: Once<TaskManager> = Once::new();

pub static ELFS_INFO: Once<ElfsInfo> = Once::new();

pub static FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();

pub static KERNEL_ADDRSPACE: Once<AddrSpace> = Once::new();

#[unsafe(link_section = ".bss.kstack")]
pub static mut KERNEL_STACK: [KernelStack; NUM_HART_MAX] = [KernelStack::ZERO; NUM_HART_MAX];

#[unsafe(link_section = ".bss.ustack")]
pub static mut USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack::ZERO; MAX_APP_NUM];

pub static ARCH: Arch = Arch::new();