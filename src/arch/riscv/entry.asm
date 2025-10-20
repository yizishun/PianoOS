    .section .text.entry
    .global _start
_start:
    /* BL33 information */
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
	/* Information end */
real_start:
    la sp, boot_stack_top
    call rust_main
    
    .section .bss.stack
    .global boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .global boot_stack_top
boot_stack_top: