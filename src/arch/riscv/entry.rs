use core::arch::naked_asm;
use crate::{config::KERNEL_STACK_SIZE_PER_HART, mm::stack::STACK};

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
            la sp, {stack}
            li t0, {per_hart_stack_size}
            addi t1, a0, 0                 //get boot hart id
            addi t1, t1, 1
         1: add sp, sp, t0
            addi t1, t1, -1
            bnez t1, 1b
            call rust_main
        ",
        stack = sym STACK,
        per_hart_stack_size = const KERNEL_STACK_SIZE_PER_HART
    )
}