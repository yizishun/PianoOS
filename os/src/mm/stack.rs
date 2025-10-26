use crate::config::STACK_SIZE;

#[repr(C, align(128))]
pub struct Stack([u8; STACK_SIZE]);

impl Stack {
    pub const ZERO: Self = Self([0; STACK_SIZE]);

    pub fn get_stack_base(&self) -> usize {
        self.0.as_ptr_range().end as usize
    }
}