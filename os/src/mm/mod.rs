use crate::{
	FrameAllocator, StackFrameAllocator, global::{FRAME_ALLOCATOR, KERNEL_ADDRSPACE}, mm::addr_space::AddrSpace
};
use alloc::boxed::Box;

pub mod heap;
pub mod stack;
pub mod address;
pub mod page_table;
pub mod frame_allocator;
pub mod addr_space;

pub fn init() {
	// init frame allocator
	FRAME_ALLOCATOR.call_once(|| FrameAllocator::new(Box::new(
		{
			// use StackFrameAllocator at first
			let mut stack = StackFrameAllocator::new();
			stack.init_scope();
			stack
		}
	)));
	// init kernel addr space
	KERNEL_ADDRSPACE.call_once(|| {
		AddrSpace::new_kernel()
	});
	// switch to translate mode
	KERNEL_ADDRSPACE.get().unwrap().activate();
}