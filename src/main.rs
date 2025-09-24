#![no_std]
#![no_main]

use core::arch::global_asm;

mod lang_items;
mod sbi;
mod console;

global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> !{
    clear_bss();
    println!("stvec {}\n", read_csr!(0x105));
    println!("Hello World\n");
    sbi::shutdown(false);
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
