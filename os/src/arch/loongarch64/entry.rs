#![cfg(target_arch = "loongarch64")]
use core::arch::naked_asm;
use crate::{config::KERNEL_STACK_SIZE_PER_HART, global::KERNEL_STACK};
use crate::arch::hart::{HART_INFO, HART_INFO_SIZE};

#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
    naked_asm!(
        "
            b 1f
            .balign 4
            .word 0x33334c42
            .word 0xdeadbeea
            .word 0xdeadbeeb
            .quad 0x80200000
            .word 0xdeadbeec
            .balign 4
        1:
            la      $sp, {stack}
            li.d    $t0, {per_hart_stack_size}
            addi.d  $t1, $a0, 0
            addi.d  $t1, $t1, 1
        2:  add.d   $sp, $sp, $t0
            addi.d  $t1, $t1, -1
            bnez    $t1, 2b

            la      $t2, {hart_info}
            li.d    $t0, {hart_info_size}
            addi.d  $t1, $a0, 0
        3:  beqz    $t1, 4f
            add.d   $t2, $t2, $t0
            addi.d  $t1, $t1, -1
            b       3b
        4:
            // 写 KS0（0x30）保存每核指针
            csrwr   $t2, 0x30

            bl      rust_main
        ",
        stack = sym KERNEL_STACK,
        per_hart_stack_size = const KERNEL_STACK_SIZE_PER_HART,
        hart_info = sym HART_INFO,
        hart_info_size = const HART_INFO_SIZE
    )
}

#[unsafe(naked)]
#[unsafe(export_name = "hart_start")]
pub unsafe extern "C" fn hart_start() -> ! {
    naked_asm!(
        "
        la      $sp, {stack}
        li.d    $t0, {per_hart_stack_size}
        addi.d  $t1, $a0, 0
        addi.d  $t1, $t1, 1
    1:  add.d   $sp, $sp, $t0
        addi.d  $t1, $t1, -1
        bnez    $t1, 1b

        la      $t2, {hart_info}
        li.d    $t0, {hart_info_size}
        addi.d  $t1, $a0, 0
    2:  beqz    $t1, 3f
        add.d   $t2, $t2, $t0
        addi.d  $t1, $t1, -1
        b       2b
    3:
        csrwr   $t2, 0x30
        bl      hart_main
        ",
        stack = sym KERNEL_STACK,
        per_hart_stack_size = const KERNEL_STACK_SIZE_PER_HART,
        hart_info = sym HART_INFO,
        hart_info_size = const HART_INFO_SIZE
    )
}
