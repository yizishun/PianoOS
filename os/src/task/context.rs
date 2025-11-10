use crate::global::__restore;

/// Task Context
/// TODO: arch specific
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
	/// return address ( e.g. __restore ) of __switch ASM function
	ra: usize,
	/// kernel stack pointer of app
	sp: usize,
	/// callee saved registers:  s 0..11
	s: [usize; 12],
	/// tp save kernel stack position for trap stage use
	tp: usize
}

impl TaskContext {
	/// init task context
	pub fn zero_init() -> Self {
		Self {
			ra: 0,
			sp: 0,
			s: [0; 12],
			tp: 0
		}
	}

	/// set task context {__restore ASM funciton, kernel stack, s_0..12 }
    	pub fn goto_restore(kstack_ptr: usize) -> Self {
		Self {
			ra: (&raw const __restore) as usize,
			sp: kstack_ptr,
			s: [0; 12],
			tp: kstack_ptr
		}
	}

}