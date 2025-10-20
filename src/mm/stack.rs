use crate::config::{KERNEL_STACK_SIZE_PER_HART, NUM_HART_MAX};

#[unsafe(link_section = ".bss.stack")]
pub static mut STACK: [Stack; NUM_HART_MAX] = [Stack::ZERO; NUM_HART_MAX];

#[repr(C, align(128))]
pub struct Stack([u8; KERNEL_STACK_SIZE_PER_HART]);

impl Stack {
    const ZERO: Self = Self([0; KERNEL_STACK_SIZE_PER_HART]);
}