pub const VIRT_FREQUNCY: i32 = 10;
pub const KERNEL_HEAP_SIZE: usize = 32 * 1024;
pub const KERNEL_STACK_SIZE: usize = 128 * 1024;
pub const USER_STACK_SIZE: usize = 4 * 1024;
pub const NUM_HART_MAX: usize = 8;
pub const MAX_APP_NUM: usize = 15;
pub const APP_BASE_ADDR: usize = 0x80400000;

// each hart should have a kernel stack,
// but kernel stack num is depend on MAX_APP_NUM
const _: () = assert!(NUM_HART_MAX <= MAX_APP_NUM);
