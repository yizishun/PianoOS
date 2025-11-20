pub const VIRT_FREQUNCY: usize = 10 * 1_000_000; //hz 即1s跑这么多周期
pub const TICK_MS: usize = 10;
pub const KERNEL_HEAP_SIZE: usize = 4 * 1024 * 1024;
pub const KERNEL_STACK_SIZE: usize = 256 * 1024;
pub const KERNEL_STACK_ALIGN: usize = 4096;
pub const USER_STACK_SIZE: usize = 4 * 1024;
pub const NUM_HART_MAX: usize = 8;
pub const MAX_APP_NUM: usize = 20;
pub const APP_BASE_ADDR: usize = 0x80a00000;

// each hart should have a kernel stack,
// but kernel stack num is depend on MAX_APP_NUM
const _: () = assert!(NUM_HART_MAX <= MAX_APP_NUM);
