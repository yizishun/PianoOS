
pub const HART_INFO_SIZE: usize = size_of::<HartContext>();

use core::ptr::NonNull;

// Make sure HartContext is aligned.
//
// HartContext will always at the end of Stack, so we should make sure
// STACK_SIZE_PER_HART is a multiple of b.
use crate::{arch::common::{ArchHarts, FlowContext}, config::STACK_SIZE, global::ARCH, mm::stack::Stack};
const _: () = assert!(STACK_SIZE % core::mem::align_of::<HartContext>() == 0);

#[repr(C, align(128))]
pub struct HartContext {
        flow_context: crate::arch::common::FlowContext,
        hartid: usize,
}

impl HartContext {
        pub fn init(&mut self, hartid: usize){
                self.hartid = hartid;
        }

        pub fn get_hartnum() -> usize {
                crate::PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
        }

        pub fn get_hartid(&self) -> usize {
                self.hartid
        }

        pub fn context_ptr(&mut self) -> NonNull<FlowContext>{
                unsafe {
                        NonNull::new_unchecked(&mut self.flow_context)
                }
        }
}

/// helper function
/// get current hart id, but call it after init HartContext in Stack and init sscratch to TrapHandler
/// it will not work when in trap handler stage, because sscratch is saving user space sp
/// when in trap handler stage, you can get hartid in ctx
pub fn current_hartid_in_boot_stage() -> usize {
        let scratch = ARCH.get_scratch();
        let stack = scratch - Stack::stack_space_size() - Stack::hart_context_size();
        let hart_context = stack as *const HartContext;
        unsafe {
                hart_context.as_ref().unwrap().get_hartid()
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