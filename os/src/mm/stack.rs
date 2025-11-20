use core::intrinsics::forget;
use core::ptr::NonNull;
use core::ops::Range;
use alloc::alloc::dealloc;
use riscv::interrupt::Trap;
use core::alloc::Layout;

use crate::arch::common::{ArchHarts, ArchPower, ArchTrap, FlowContext};
use crate::global::ARCH;
use crate::arch::common::Arch;
use crate::{harts::HartContext, config::{USER_STACK_SIZE, KERNEL_STACK_SIZE}};
use crate::trap::{FreeTrapStack, TrapHandler};
use crate::trap::fast::FastHandler;
use crate::config::KERNEL_STACK_ALIGN;

// Make sure stack address can be aligned.
const _: () = assert!(KERNEL_STACK_SIZE % align_of::<KernelStack>() == 0);

// Make sure alignment of TrapHandler is smaller than Stack
const _: () = assert!(align_of::<KernelStack>() >= align_of::<TrapHandler>());

#[repr(C, align(128))]
pub struct UserStack([u8; USER_STACK_SIZE]);

impl UserStack {
    	pub const ZERO: Self = Self([0; USER_STACK_SIZE]);
	
	pub fn as_ptr_range(&self) -> core::ops::Range<*const u8>{
		self.0.as_ptr_range()
	}
}

#[repr(C, align(128))]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

//                      Stack
//     low_addr   +----HartContext---+
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
impl KernelStack {
    	pub const ZERO: Self = Self([0; KERNEL_STACK_SIZE]);

	pub fn as_ptr_range(&self) -> core::ops::Range<*const u8>{
		self.0.as_ptr_range()
	}

	/// get hart context size
	pub const fn hart_context_size() -> usize {
		size_of::<HartContext>()
	}

	/// get trap handler size
	pub const fn trap_handler_size() -> usize {
		KERNEL_STACK_SIZE -
			(KERNEL_STACK_SIZE - size_of::<TrapHandler>()) & !(align_of::<TrapHandler>() - 1)
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

	pub fn init_trap_stack(&'static mut self, hartid: usize, flow_context: NonNull<FlowContext>, fast_handler: FastHandler) -> FreeTrapStack{
		let hart_context = self.hart_context_mut();
		let context_ptr = flow_context;
		let hart_ptr = unsafe { NonNull::new_unchecked(hart_context) };
		hart_context.init(hartid);

		let range = self.0.as_ptr_range();
		FreeTrapStack::new(
			range.start as usize.. range.end as usize, 
			|_| {}, 
			context_ptr,
			hart_ptr,
			fast_handler
		).unwrap()
	}

	/// Initializes stack for trap handling.
    	/// - Sets up hart context.
    	/// - Creates and loads FreeTrapStack with the stack range.
    	pub fn load_as_stack(
		&'static mut self, 
		hartid: usize, 
		flow_context: NonNull<FlowContext>, 
		fast_handler: FastHandler,
		drop: fn(Range<usize>),
	) {
		let hart_context = self.hart_context_mut();
		let context_ptr = flow_context;
		let hart_ptr = unsafe { NonNull::new_unchecked(hart_context) };
		hart_context.init(hartid);

		let range = self.0.as_ptr_range();
		forget(
			FreeTrapStack::new(
				range.start as usize.. range.end as usize, 
				drop,
				context_ptr,
				hart_ptr,
				fast_handler
			).unwrap().load()
		);
	}
}

/// drop kernel stack and internel flow_context
pub fn stack_drop(range: Range<usize>) {
	assert_eq!(range.end - range.start, KERNEL_STACK_SIZE);
	let trap_handler = (range.end - KernelStack::trap_handler_size()) as *mut TrapHandler;
	let flow_context = unsafe { (*trap_handler).context };
	let stack_layout = Layout::from_size_align(range.end - range.start, KERNEL_STACK_ALIGN).unwrap();
	let flow_context_layout = Layout::new::<FlowContext>();
	unsafe {
		dealloc(range.start as *mut u8, stack_layout);
		dealloc(flow_context.as_ptr() as *mut u8, flow_context_layout);
	}
}
