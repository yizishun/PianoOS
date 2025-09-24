#![no_std]
#![no_main]

use core::arch::global_asm;
use riscv::register::{stvec, mtvec};

mod lang_items;
mod sbi;
mod console;

global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> !{
    clear_bss();
    let stvec_value= stvec::read().bits();
    println!("mtvec = {}", stvec_value);
    println!("Hello World");
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
