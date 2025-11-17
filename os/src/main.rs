#![no_std]
#![no_main]
#![allow(named_asm_labels)]
#![feature(ptr_mask)]
#![feature(core_intrinsics)]
#![feature(generic_atomic)]
#![feature(sync_unsafe_cell)]
#![feature(min_specialization)]
#![feature(stmt_expr_attributes)]

use log::info;

use crate::arch::common::ArchPower;
use crate::global::*;
use crate::loader::Loader;
use crate::logging::PIANOLOGGER;
use crate::logging::PianoLogger;
use crate::{
	harts::HartContext, task::TaskManager, mm::heap::heap_init, platform::Platform,
};

mod arch;
mod task;
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
mod syscall;
mod loader;

extern crate alloc;

#[unsafe(no_mangle)]
extern "C" fn rust_main(hartid: usize, device_tree: usize) -> ! {
	clear_bss();
	heap_init();

	PLATFORM.call_once(|| Platform::init_platform(device_tree).unwrap());

	PIANOLOGGER.call_once(|| { PianoLogger::set_boot_logger() });
	logging::init().expect("Logging System init fail");
	info!("Logging system init success");
	info!("boot hartid: {}", hartid);
	info!("device tree addr: {:p}", device_tree as *const u8);
	PLATFORM.get().unwrap().print_platform_info();
	print_kernel_mem();

	LOADER.call_once(|| Loader::new());
	TASK_MANAGER.call_once(|| 
		TaskManager::new()
	);
	let next_app = 
		//init HartContext, TrapContext, TaskContext
		TASK_MANAGER.get().unwrap().prepare_next_at_boot(hartid);

	LOADER.get().unwrap().print_app_info();

	// elf load happen in this func
	if TASK_MANAGER.get().unwrap().num_app != 0 {
		//  switch logger
		PIANOLOGGER.get().unwrap().set_trap_logger();
		for i in 0..HartContext::get_hartnum() {
			let start_addr = arch::common::entry::hart_start as usize;
			sbi_rt::hart_start(i, start_addr, 0); //TODO: arch specific
		}

		TASK_MANAGER
			.get()
			.unwrap()
			.run_next_at_boot(next_app)
	} else {
		info!("No app should be run, kernel shutdown");
		ARCH.shutdown(false);
	}

	unreachable!();
}

#[unsafe(no_mangle)]
extern "C" fn hart_main(hartid: usize, _opaque: usize) -> ! {
	let next_app = 
		//init HartContext, TrapContext, TaskContext
		TASK_MANAGER.get().unwrap().prepare_next_at_boot(hartid);

	TASK_MANAGER
		.get()
		.unwrap()
		.run_next_at_boot(next_app);

	unreachable!();
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
		info!(".text      : [{:<10p}, {:<10p}]", &stext, &etext);
		info!(".rodata    : [{:<10p}, {:<10p}]", &srodata, &erodata);
		info!(".data      : [{:<10p}, {:<10p}]", &sdata, &edata);
		info!(".bss.kstack: [{:<10p}, {:<10p}]", &skstack, &ekstack);
		info!(".bss.ustack: [{:<10p}, {:<10p}]", &sustack, &eustack);
		info!(".bss.heap  : [{:<10p}, {:<10p}]", &sheap, &eheap);
		info!(".bss       : [{:<10p}, {:<10p}]", &sbss, &ebss);
		info!("kernel end = {:<10p}", &ekernel);
	}
}
