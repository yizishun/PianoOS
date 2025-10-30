use crate::{arch::common::hart::HartContext, config::STACK_SIZE};

#[repr(C, align(128))]
pub struct Stack([u8; STACK_SIZE]);

//                      Stack
//     low_addr   +----HartContext---+
//                |  flowContext     |
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
//     hign addr  +------------------+
impl Stack {
    	pub const ZERO: Self = Self([0; STACK_SIZE]);

    	/// get mut hartContext in stack
    	pub fn hart_context_mut(&mut self) -> &mut HartContext {
        	unsafe { &mut *self.0.as_mut_ptr().cast() }
    	}

    	/// get hartContext in stack
    	pub fn hart_context(&mut self) -> & HartContext {
        	unsafe { & *self.0.as_mut_ptr().cast() }
    	}
}
