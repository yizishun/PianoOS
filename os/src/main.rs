#![no_std]
#![no_main]

use core::arch::global_asm;
use log::{info};
use spin::Once;

use crate::{arch::common::hart::HartInfo, batch::AppManager, mm::heap::heap_init, platform::Platform};
use crate::global::*;
use crate::config::NUM_HART_MAX;

mod config;
mod global;
mod driver;
mod console;
mod error;
mod arch;
mod logging;
mod devicetree;
mod platform;
mod mm;
mod macros;
mod batch;

extern crate alloc;

static BOOT_HARTID: Once<usize> = spin::Once::new();

#[unsafe(no_mangle)]
extern "C" fn rust_main(hartid: usize, device_tree: usize) -> ! {
    // 1. get boot hartid and device tree addr 
    BOOT_HARTID.call_once(|| hartid);
    // 2. clear bss, heap init and hart info init
    clear_bss();
    heap_init();
    HART_INFO.call_once(|| {
        let mut hart_info = [HartInfo::ZERO_HART; NUM_HART_MAX];
        for (i, h) in (&mut hart_info).iter_mut().enumerate() {
            *h = HartInfo::new(i);
        }
        hart_info
    });
    APP_MANAGER.call_once(|| AppManager::new());
    // 3. parse device tree and init platform
    PLATFORM.call_once(|| {
        Platform::init_platform(device_tree)
            .unwrap()
    });
    // 4. logging system init and print some infomation
    logging::init().expect("Logging System init fail");
    info!("Logging system init success");
    info!("boot hartid: {}", hartid);
    info!("device tree addr: {:p}", device_tree as *const u8);
    PLATFORM.get().unwrap().print_platform_info();
    // 5. boot hart start other harts
    let cur_hart = HartInfo::get_cur_hart();
    for i in 0..HartInfo::get_hartnum() {
        let start_addr = arch::common::entry::hart_start as usize;
        sbi_rt::hart_start(i, start_addr, 0);
    }
    // 6. print some kernel info and app info
    print_kernel_mem();
    info!("kernel current hart state: {}", cur_hart.get_cur_hart_state());
    (0..HartInfo::get_hartnum()).for_each(|id|{
        info!("hart{}: {}", id, arch::common::hart::get_hart_state(id))
    });
    APP_MANAGER.get().unwrap().print_app_info();
    // 7. boot hart shutdown
    arch::common::shutdown(false);
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
