#![no_std]
#![no_main]

use core::arch::global_asm;

mod lang_items;

global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> !{
    clear_bss();
    loop {};
}

fn clear_bss() {
    unsafe extern "C" {
        static sbss: usize;
        static ebss: usize;
    }
    unsafe {
        for i in sbss..ebss{
            *(i as *mut u8) = 0;
        }
    }

}
