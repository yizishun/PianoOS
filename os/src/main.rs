#![no_std]
#![no_main]
#![feature(ptr_mask)]

use core::arch::global_asm;
use log::info;
use spin::Once;

use crate::arch::common::ArchPower;
use crate::config::NUM_HART_MAX;
use crate::global::*;
use crate::{
        harts::HartContext, batch::AppManager, mm::heap::heap_init, platform::Platform,
};

mod arch;
mod batch;
mod config;
mod console;
mod devicetree;
mod driver;
mod error;
mod global;
mod logging;
mod macros;
mod mm;
mod platform;
mod trap;
mod harts;

extern crate alloc;

static BOOT_HARTID: Once<usize> = spin::Once::new();

#[unsafe(no_mangle)]
extern "C" fn rust_main(hartid: usize, device_tree: usize) -> ! {
        // 1. get boot hartid and device tree addr
        BOOT_HARTID.call_once(|| hartid);
        // 2. clear bss, heap init and hart info init
        clear_bss();
        heap_init();
        APP_MANAGER.call_once(|| AppManager::new());
        // 3. parse device tree and init platform
        PLATFORM.call_once(|| Platform::init_platform(device_tree).unwrap());
        // 4. logging system init and print some infomation
        logging::init().expect("Logging System init fail");
        info!("Logging system init success");
        info!("boot hartid: {}", hartid);
        info!("device tree addr: {:p}", device_tree as *const u8);
        PLATFORM.get().unwrap().print_platform_info();
        // 5. boot hart start other harts
        for i in 0..HartContext::get_hartnum() {
                let start_addr = arch::common::entry::hart_start as usize;
                sbi_rt::hart_start(i, start_addr, 0);
        }
        // 6. print some kernel info and app info
        print_kernel_mem();
        APP_MANAGER.get().unwrap().print_app_info();
        // 7. boot hart shutdown
        info!("Kernel shutdown");
        ARCH.shutdown(false);
}

#[unsafe(no_mangle)]
extern "C" fn hart_main(hartid: usize, opaque: usize) -> ! {
        info!("hart {} boot success", hartid);
        loop {}
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
                info!(".bss.stack: [{:<10p}, {:<10p}]", &sstack, &estack);
                info!(".bss.heap : [{:<10p}, {:<10p}]", &sheap, &eheap);
                info!(".bss      : [{:<10p}, {:<10p}]", &sbss, &ebss);
                info!("kernel end = {:<10p}", &ekernel);
        }
}
