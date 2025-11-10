use core::arch::asm;
use core::ops::Range;
use core::ptr::null;
use core::sync::atomic::Ordering;
use core::array;

use alloc::task;
use log::info;
use spin::mutex::Mutex;

use crate::arch::riscv::trap::fast_switch;
use crate::global::{ARCH, KERNEL_STACK, LOADER, USER_STACK};
use crate::arch::common::{ArchPower, ArchTrap, boot_entry, boot_handler, switch};
use crate::config::{MAX_APP_NUM, USER_STACK_SIZE};
use crate::harts::{hart_context_in_boot_stage, hart_context_in_trap_stage};
use crate::mm::stack::UserStack;
use crate::task::block::TaskControlBlock;
use crate::task::context::TaskContext;
use crate::task::status::TaskStatus;

pub mod harts;
pub mod block;
pub mod status;
pub mod context;

pub struct TaskManager {
	pub num_app: usize,
	pub app_range: [Range<*const u8>; MAX_APP_NUM],
	finished: Mutex<bool>,
	tasks: [TaskControlBlock; MAX_APP_NUM] 
}

unsafe impl Send for TaskManager {}
unsafe impl Sync for TaskManager {}

impl TaskManager {
	pub fn new() -> Self {
		let num_app = LOADER.get().unwrap().num_app;
		// load app and init app_range
		let app_range: [Range<*const u8>; MAX_APP_NUM] = 
			array::from_fn(|i| {
				if i < num_app {
					LOADER.get().unwrap()
						.load(i)
				} else {
					Range { start: null(), end:  null()}
				}
			});
		// init kernel stacks and taskControlBlocks
		let tasks: [TaskControlBlock; MAX_APP_NUM] = 
			array::from_fn(|i| {
				let kstack_ptr = unsafe {
					let free_stack= KERNEL_STACK[i].init_trap_stack(i);
					free_stack.kstack_ptr()
				};
				TaskControlBlock::new(kstack_ptr)
			});
		TaskManager { 
			num_app: num_app,
			finished: Mutex::new(false),
			app_range,
			tasks
		}
	}

	pub fn app_size(&self, app_id: usize) -> usize {
		let app_range = self.app_range.get(app_id).unwrap();
		let start = app_range.start as usize;
		let end = app_range.end as usize;
		end - start
	}

	pub fn check_end(&self) {
		let all_finished = 
			self.tasks.iter().take(self.num_app)
				.all(|f| f.status() == TaskStatus::Exited);
		if all_finished {
			*self.finished.lock() = true;
			info!("All applications completed! Kennel shutdown");
			ARCH.shutdown(false);
		} else {
			//info!("Waiting other program finished");
			//self.finished.iter().take(self.num_app).enumerate().for_each(|e| {
				//info!("program({:?}) finished: {:?}", e.0, e.1);
			//});
		}
	}

	/// this will find next ready app and set running(use CAS)
	fn find_next_ready_and_set_run(&self) -> usize {
		loop {
			for (id, task) in self.tasks.iter().enumerate() {
				if task.task_status.compare_exchange(
					TaskStatus::Ready as u8,
					TaskStatus::Running as u8,
					Ordering::Acquire,
					Ordering::Relaxed).is_ok() {
					return id;
				}
			}
			self.check_end();
		}
	}


	pub fn run_next_at_boot(&self) -> !{
		let next_app = self.find_next_ready_and_set_run();
		let hart_context = hart_context_in_boot_stage();
		// init hart app info
		let app_range = &self.app_range[next_app];
		hart_context.app_info.start(next_app, app_range.clone());
		// init sepc, sstatus, stvec
		boot_handler(app_range.start as usize);
		// init user stack and sret
		unsafe {
			boot_entry(hart_context.hartid())
		}
	}

	pub fn fast_run_next_at_trap(&self) {
		let next_app = self.find_next_ready_and_set_run();
		let old_hart_context = hart_context_in_trap_stage();
		// switch task context
		// set new tp, sp, s0-s11, ra
		unsafe {
			fast_switch(
				&self.tasks.get(next_app).unwrap().task_cx as *const TaskContext
			);
			// set new caller saved reg by compiler
			asm!(
				".global __restore",
				"__restore: "
			)
		}
		// the switch will set new tp so we should capture new hart context
		let new_hart_context = hart_context_in_trap_stage();
		// init new hartid
		new_hart_context.set_hartid(old_hart_context.hartid());
		// init new hart app info
		let app_range = &self.app_range[next_app];
		new_hart_context.app_info.start(next_app, app_range.clone());
		info!("Kernel loading app({})", next_app);
		// set sepc, sscratch, sstatus
		unsafe {
			// sepc
			ARCH.set_next_pc(app_range.start as usize);
			// sscratch
			#[allow(static_mut_refs)]
			let user_stack = USER_STACK.get(next_app).unwrap() 
				as *const UserStack as usize;
			ARCH.set_next_user_stack(user_stack + USER_STACK_SIZE);
			// sstatus still
		}
	}

	pub fn run_next_at_trap(&self) {
		let next_app = self.find_next_ready_and_set_run();
		let old_hart_context = hart_context_in_trap_stage();
		let cur_app = old_hart_context.app_info.cur_app;
		// switch task context
		// set new tp, sp, s0-s11, ra
		unsafe {
			switch(
				&self.tasks.get(cur_app).unwrap().task_cx as *const TaskContext,
				&self.tasks.get(next_app).unwrap().task_cx as *const TaskContext
			);
			// set new caller saved reg by compiler
			asm!(
				".global __restore",
				"__restore: "
			)
		}
		// the switch will set new tp so we should capture new hart context
		let new_hart_context = hart_context_in_trap_stage();
		// init new hartid
		new_hart_context.set_hartid(old_hart_context.hartid());
		// init new hart app info
		let app_range = &self.app_range[next_app];
		new_hart_context.app_info.start(next_app, app_range.clone());
		info!("Kernel loading app({})", next_app);
		// set sepc, sscratch, sstatus
		unsafe {
			// sepc
			ARCH.set_next_pc(app_range.start as usize);
			// sscratch
			#[allow(static_mut_refs)]
			let user_stack = USER_STACK.get(next_app).unwrap() 
				as *const UserStack as usize;
			ARCH.set_next_user_stack(user_stack + USER_STACK_SIZE);
			// sstatus still
		}
	}

	pub fn exit_cur_and_run_next(&self) {
		let app_id = hart_context_in_trap_stage().app_info.cur_app;
		let task_block = self.tasks.get(app_id).unwrap();
		assert!(task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		task_block.mark_exit();
		self.fast_run_next_at_trap();
	}

	pub fn suspend_cur_and_run_next(&self) {
		let app_id = hart_context_in_trap_stage().app_info.cur_app;
		let task_block = self.tasks.get(app_id).unwrap();
		assert!(task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		task_block.mark_suspend();
		self.run_next_at_trap();
	}
}
