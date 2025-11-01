use core::intrinsics::forget;

use crate::arch::common::ArchHarts;
use crate::global::ARCH;
use crate::{harts::HartContext, config::STACK_SIZE};
use crate::trap::{FreeTrapStack, TrapHandler};
use crate::arch::common::fast_handler;

// Make sure stack address can be aligned.
const _: () = assert!(STACK_SIZE % align_of::<Stack>() == 0);

// Make sure alignment of TrapHandler is smaller than Stack
const _: () = assert!(align_of::<Stack>() >= align_of::<TrapHandler>());

#[repr(C, align(128))]
pub struct Stack([u8; STACK_SIZE]);

//                      Stack
//     low_addr   +----HartContext---+
//                |  flow_context    |
//                |  hart_id         |
//                +----Stack Space---+
//                |                  |
//                |                  |
//                |                  |
//                +----TrapHandler---+
//           sp-> | context(ptr)     |
//                | fast_handler(ptr)|
//                | scratch          |
//                | range            |
//                | drop(ptr)        |
//                | ...(unalign)     |
//     hign addr  +------------------+
impl Stack {
    	pub const ZERO: Self = Self([0; STACK_SIZE]);

	/// get hart context size
	pub const fn hart_context_size() -> usize {
		size_of::<HartContext>()
	}

	/// get trap handler size
	pub const fn trap_handler_size() -> usize {
		STACK_SIZE -
			(STACK_SIZE - size_of::<TrapHandler>()) & !(align_of::<TrapHandler>() - 1)
	}

	pub const fn stack_space_size() -> usize {
		size_of::<Self>() - Self::trap_handler_size() - Self::hart_context_size()
	}

	/// get current Stack struct
	/// should set sscratch point to stack space before use it
	pub unsafe fn current_stack() -> &'static Self {
		let scratch = ARCH.get_scratch();
		let stack_space_ptr = scratch as *const u8;
		let stack_ptr = unsafe { 
			stack_space_ptr.byte_sub(
				Self::stack_space_size() + Self::hart_context_size())
		};
		unsafe {
			& *stack_ptr.cast()
		}
		
	}

	/// get current Stack mut struct
	/// should set sscratch point to stack space before use it
	pub unsafe fn current_stack_mut() -> &'static mut Self {
		let scratch = ARCH.get_scratch();
		let stack_space_ptr = scratch as *mut u8;
		let stack_ptr = unsafe { 
			stack_space_ptr.byte_sub(
				Self::stack_space_size() + Self::hart_context_size())
		};
		unsafe {
			&mut *stack_ptr.cast()
		}
		
	}

    	/// get mut hartContext in stack
    	pub fn hart_context_mut(&mut self) -> &mut HartContext {
        	unsafe { &mut *self.0.as_mut_ptr().cast() }
    	}

    	/// get hartContext in stack
    	pub fn hart_context(&mut self) -> &HartContext {
        	unsafe { & *self.0.as_mut_ptr().cast() }
    	}

	/// Initializes stack for trap handling.
    	/// - Sets up hart context.
    	/// - Creates and loads FreeTrapStack with the stack range.
    	pub fn load_as_stack(&'static mut self, hartid: usize) {
		let hart_context = self.hart_context_mut();
		let context_ptr = hart_context.context_ptr();
		hart_context.init(hartid);

		let range = self.0.as_ptr_range();
		forget(
			FreeTrapStack::new(
				range.start as usize.. range.end as usize, 
				|_| {}, 
				context_ptr,
				fast_handler
			).unwrap().load()
		);
	}
}
