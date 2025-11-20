use core::ops::Range;

use crate::config::KERNEL_HEAP_SIZE;
use alloc::alloc::dealloc;
use buddy_system_allocator::LockedHeap;

#[unsafe(link_section = ".bss.heap")]
static mut HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeap<20> = LockedHeap::<20>::empty();

pub fn heap_init() {
	#[allow(static_mut_refs)]
	unsafe {
		HEAP_ALLOCATOR.lock()
			      .init(HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
	}
}
