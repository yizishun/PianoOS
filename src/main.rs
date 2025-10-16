#![no_std]
#![no_main]

use core::arch::global_asm;
use log::{info};

use crate::{mm::heap::heap_init, platform::PLATFORM};

mod driver;
mod console;
mod lang_items;
mod sbi;
mod logging;
mod devicetree;
mod platform;
mod mm;
mod macros;

extern crate alloc;

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
    static sheap: usize;
    static eheap: usize;
    static sbss: usize;
    static ebss: usize;
    static ekernel: usize;
}

const FREQUNCY: i32 = 10;
static mut BOOT_HARTID: usize = 0;

#[unsafe(no_mangle)]
extern "C" fn rust_main(hartid: usize, device_tree: usize) -> ! {
    // 1. get boot hartid and device tree addr 
    // SAFETY: boot_hartid will be assign once at boot time
    unsafe {
        BOOT_HARTID = hartid;
    }
    // SAFETY: PLATFORM infomation will be init once
    #[allow(static_mut_refs)]
    unsafe { 
        PLATFORM.init(device_tree); 
        info!("Cpu Number: {}", PLATFORM.board_info.cpu_num.unwrap());
    }
    // 2. clear bss and heap init
    clear_bss();
    heap_init();
    // 3. boot hart init loging system
    logging::init().expect("Logging System init fail");
    info!("1.Logging system init success ------");
    info!("boot hartid: {}", hartid);
    info!("device tree addr: {:p}", device_tree as *const u8);
    // 4. boot hart prepare env for all harts
    // 5. boot hart start other harts
    // 6. print some kernel information
    print_kernel_mem();
    info!("kernel hart number: {}", sbi::hart::get_hartnum());
    info!("kernel current hart state: {}", sbi::hart::get_cur_hart_state());
    (0..sbi::hart::get_hartnum()).for_each(|id|{
        info!("hart{}: {}", id, sbi::hart::get_hart_state(id))
    });
    // 7. boot hart shutdown
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
        info!(".bss.heap : [{:<10p}, {:<10p}]", &sheap, &eheap);
        info!(".bss      : [{:<10p}, {:<10p}]", &sbss, &ebss);
        info!("kernel end = {:<10p}", &ekernel);
    }
}
