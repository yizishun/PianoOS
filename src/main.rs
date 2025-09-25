#![no_std]
#![no_main]

use core::arch::global_asm;

use crate::sbi::sleep;
const FREQUNCY: i32 = 10;

mod console;
mod lang_items;
mod sbi;

global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello World");
    sleep(5);
    let a = 1;
    error!("1 is {}", a);
    info!("1 is {}", a);
    sbi::shutdown(false);
}

fn clear_bss() {
    unsafe extern "C" {
        static sbss: usize;
        static ebss: usize;
    }
    unsafe {
        for i in sbss..ebss {
            *(i as *mut u8) = 0;
        }
    }
}
