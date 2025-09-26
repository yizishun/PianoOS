#![no_std]
#![no_main]

use core::arch::global_asm;
use log::{info};

const FREQUNCY: i32 = 10;

mod console;
mod lang_items;
mod sbi;
mod logging;

global_asm!(include_str!("entry.asm"));
unsafe extern "C" {
    static skernel: usize;
    static stext: usize;
    static etext: usize;
    static srodata: usize;
    static erodata: usize;
    static sdata: usize;
    static edata: usize;
    static boot_stack_lower_bound: usize;
    static boot_stack_top: usize;
    static sbss: usize;
    static ebss: usize;
    static ekernel: usize;
}

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init().expect("Logging System init fail");
    print_kernel_mem();
    sbi::shutdown(false);
}

fn clear_bss() {
    unsafe {
        let mut ptr = &raw const sbss as *mut u8;
        let end = &raw const ebss as *mut u8;
        while ptr < end {
            ptr.write_volatile(0);
            ptr = ptr.offset(1);
        }
    }
}

fn print_kernel_mem() {
    unsafe {
        info!("kernel memory map:");
        info!("kernel base = {:<10p}", &skernel);
        info!(".text     : [{:<10p}, {:<10p}]", &stext, &etext);
        info!(".rodata   : [{:<10p}, {:<10p}]", &srodata, &erodata);
        info!(".data     : [{:<10p}, {:<10p}]", &sdata, &edata);
        info!(".bss.stack: [{:<10p}, {:<10p}]", &boot_stack_lower_bound, &boot_stack_top);
        info!(".bss      : [{:<10p}, {:<10p}]", &sbss, &ebss);
        info!("kernel end = {:<10p}", &ekernel);
    }
}
