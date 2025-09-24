#![no_std]
#![no_main]

use core::arch::global_asm;
use riscv::register::{sie};

use crate::sbi::sleep;
const FREQUNCY: i32 = 10;

mod lang_items;
mod sbi;
mod console;

global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> !{
    clear_bss();
    println!("Hello World");
    sleep(5);
    println!("sie = {}", sie::read().bits());
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
