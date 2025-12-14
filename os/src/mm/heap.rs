use crate::config::KERNEL_HEAP_SIZE;
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

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
	panic!("Heap allocation error, layout = {:?}", layout);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test_case]
	fn test_heap_allocation() {
		use alloc::boxed::Box;
		let heap_value = Box::new(42);
		assert_eq!(*heap_value, 42);
		crate::println!("test_heap_allocation passed!");
	}

	#[test_case]
	fn test_vec_allocation() {
		use alloc::vec::Vec;
		let mut vec = Vec::new();
		for i in 0..500 {
			vec.push(i);
		}
		assert_eq!(vec.len(), 500);
		for i in 0..500 {
			assert_eq!(vec[i], i);
		}
		crate::println!("test_vec_allocation passed!");
	}
}
