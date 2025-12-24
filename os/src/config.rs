pub const VIRT_FREQUNCY: usize = 10 * 1_000_000; //hz 即1s跑这么多周期
pub const TICK_MS: usize = 1;
pub const KERNEL_HEAP_SIZE: usize = 4 * 1024 * 1024;
pub const KERNEL_STACK_SIZE: usize = 1024 * 1024;
pub const KERNEL_STACK_ALIGN: usize = 4096;
pub const USER_STACK_SIZE: usize = 4 * 1024;
pub const NUM_HART_MAX: usize = 8;
pub const MAX_APP_NUM: usize = 20;
pub const APP_BASE_ADDR: usize = 0x80a00000; //TODO: remove it
pub const APP_VIRT_ADDR: usize = 0x1000;
pub const TRAMPOLINE_VADDR: usize = usize::MAX - PAGE_SIZE + 1;
pub const FLOW_CONTEXT_VADDR: usize = TRAMPOLINE_VADDR - PAGE_SIZE;
pub const TRAP_HANDLER_VADDR: usize = TRAMPOLINE_VADDR - 2*PAGE_SIZE;
pub const PAGE_SIZE: usize = 4 * 1024; //4k page size
pub const PAGE_SIZE_BITS: usize = PAGE_SIZE.trailing_zeros() as usize;
pub const MEMORY_END: usize = 0x8200_0000;

// each hart should have a kernel stack,
// but kernel stack num is depend on MAX_APP_NUM
const _: () = assert!(NUM_HART_MAX <= MAX_APP_NUM);
