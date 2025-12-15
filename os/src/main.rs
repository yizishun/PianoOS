#![no_std]
#![no_main]
#![allow(named_asm_labels)]
#![feature(ptr_mask)]
#![feature(core_intrinsics)]
#![feature(generic_atomic)]
#![feature(sync_unsafe_cell)]
#![feature(min_specialization)]
#![feature(stmt_expr_attributes)]
#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use alloc::boxed::Box;
use log::info;

use crate::arch::common::ArchPower;
use crate::arch::common::ArchHarts;
use crate::global::*;
use crate::loader::Loader;
use crate::logging::PIANOLOGGER;
use crate::logging::PianoLogger;
use crate::mm::frame_allocator::FrameAllocator;
use crate::mm::frame_allocator::StackFrameAllocator;
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
mod test;

extern crate alloc;

#[unsafe(no_mangle)]
extern "C" fn rust_main(hartid: usize, device_tree: usize) -> ! {
	clear_bss();
	heap_init();

	// parse dtb and init platform
	PLATFORM.call_once(|| Platform::init_platform(device_tree).unwrap());

	// init log system
	PIANOLOGGER.call_once(|| { PianoLogger::set_boot_logger() });
	logging::init().expect("Logging System init fail");
	info!("Logging system init success");
	info!("boot hartid: {}", hartid);
	info!("device tree addr: {:p}", device_tree as *const u8);
	PLATFORM.get().unwrap().print_platform_info();
	crate::mm::addr_space::print_kernel_mem();

	// init frame allocator
	FRAME_ALLOCATOR.call_once(|| FrameAllocator::new(Box::new(
		{
			// use StackFrameAllocator at first
			let mut stack = StackFrameAllocator::new();
			stack.init_scope();
			stack
		}
	)));
	
	// get elf info and init loader
	LOADER.call_once(|| Loader::new());
	LOADER.get().unwrap().print_app_info();
	// elf load happen in this func
	TASK_MANAGER.call_once(|| 
		TaskManager::new()
	);
	let next_app = 
		//init HartContext, TrapContext, TaskContext
		TASK_MANAGER.get().unwrap().prepare_next_at_boot(hartid);

	//test
	#[cfg(test)]
    	test_main();

	if TASK_MANAGER.get().unwrap().num_app != 0 {
		//  switch logger
		PIANOLOGGER.get().unwrap().set_trap_logger();
		for i in 0..HartContext::get_hartnum() {
			let start_addr = arch::common::entry::hart_start as *const () as usize;
			ARCH.hart_start(i, start_addr, 0);
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
