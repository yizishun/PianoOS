#![cfg(target_arch = "riscv64")]
use crate::{config::STACK_SIZE, global::KERNEL_STACK, trap::TrapHandler};
use core::arch::naked_asm;

/// Move stack to keep a space for TrapHandler
#[unsafe(naked)]
pub unsafe extern "C" fn reuse_stack_for_trap() {
    core::arch::naked_asm!(
	"   addi sp, sp, {size}
	    andi sp, sp, {mask}
	    ret
	",
	size = const -(size_of::<TrapHandler>() as isize),
	mask = const !(align_of::<TrapHandler>() as isize - 1) ,
    )
}

/// Locates and initializes stack for each hart.
///
/// This is a naked function that sets up the stack pointer based on hart ID.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn locate() {
    core::arch::naked_asm!(
	"   la   sp, {stack}            // Load stack base address
	    li   t0, {per_hart_stack_size} // Load stack size per hart
	    mv t1, a0                   // Get current hart ID
	    addi t1, t1,  1             // Add 1 to hart ID
	 1: add  sp, sp, t0             // Calculate stack pointer
	    addi t1, t1, -1             // Decrement counter
	    bnez t1, 1b                 // Loop if not zero
	    call t1, {move_stack}       // Call stack reuse function
	    ret                         // Return
	",
	per_hart_stack_size = const STACK_SIZE,
	stack               =   sym KERNEL_STACK,
	move_stack          =   sym reuse_stack_for_trap,
    )
}

#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
	naked_asm!(
		// BL33 information
		"
		j real_start
		.balign 4
		.word 0x33334c42  /* b'BL33' */
		.word 0xdeadbeea  /* CKSUM */
		.word 0xdeadbeeb  /* SIZE */
		.quad 0x80200000  /* RUNADDR */
		.word 0xdeadbeec
		.balign 4
		j real_start
		.balign 4
		",
	"real_start:
		call {locate}	
		call rust_main
		",
		locate = sym locate
	)
}

#[unsafe(naked)]
#[unsafe(export_name = "hart_start")]
pub unsafe extern "C" fn hart_start() -> ! {
	naked_asm!(
	"hart_real_start:
		call {locate}
		call hart_main
		",
		locate = sym locate,
	)
}
