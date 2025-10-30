use crate::config::STACK_SIZE;

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

        pub fn get_stack_base(&self) -> usize {
                self.0.as_ptr_range().end as usize
        }
}
