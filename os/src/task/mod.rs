use core::ops::Range;
use core::ptr::{NonNull, null};
use core::sync::atomic::Ordering;
use core::{array, num};

use log::info;
use spin::mutex::Mutex;

use crate::global::{ARCH, KERNEL_STACK, LOADER};
use crate::arch::common::{ArchPower, ArchTime, FlowContext, boot_entry, boot_handler};
use crate::config::{MAX_APP_NUM, TICK_MS};
use crate::harts::{task_context_in_trap_stage, trap_handler_in_trap_stage};
use crate::task::block::TaskControlBlock;
use crate::task::status::{ReadyLevel, TaskStatus};

pub mod harts;
pub mod block;
pub mod status;

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
				if i < num_app {
					TaskControlBlock::new(
						i, 
						app_range[i].start as usize, 
						app_range[i].end as usize,
						TaskStatus::Ready(ReadyLevel::High))
				} else {
					TaskControlBlock::new(i, 0, 0, TaskStatus::Exited)
				}
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
			let mut lock = self.finished.lock();
			info!("All applications completed! Kennel shutdown");
			*lock = true;
			ARCH.shutdown(false);
		}
	}

	/// this will find next ready app and set running(use CAS)
	fn find_next_ready_and_set_run(&self) -> usize {
		loop {
			use ReadyLevel::*;
			let mut best_id: Option<usize> = None;
			let mut best_level = Low;
			for (id, task) in self.tasks.iter().enumerate() {
				let status = TaskStatus::try_from(task.task_status.load(Ordering::Relaxed)).unwrap();
				if let TaskStatus::Ready(level) = status {
					if best_id.is_none() || level > best_level {
						best_id = Some(id);
						best_level = level;
					}
				}

			}
			if let Some(id) = best_id {
				if self.tasks[id].task_status.compare_exchange(
					u8::from(TaskStatus::Ready(best_level)),
					u8::from(TaskStatus::Running),
					Ordering::Acquire,
					Ordering::Relaxed).is_ok() {
					assert!(self.tasks[id].status() == TaskStatus::Running);
					return id;
				}
			}
			self.check_end();
		}
	}


	pub fn prepare_next_at_boot(&self, hartid: usize) -> usize {
		let next_app = self.find_next_ready_and_set_run();
		let next_task_context = &self.tasks[next_app];
		let next_flow_context = unsafe {
			NonNull::new_unchecked(next_task_context.flow_context.get() as *mut _)
		};
		let next_app_range = &self.app_range[next_app];
		
		// init trap stack, task context, hart context and bind to an app
		unsafe {
			#[allow(static_mut_refs)]
			KERNEL_STACK.get_mut(hartid).unwrap()
				.load_as_stack(hartid, next_flow_context);	
		}
		// init sepc, sstatus, stvec, stie
		boot_handler(next_app_range.start as usize);
		next_app
	}

	pub fn run_next_at_boot(&self, next_app: usize) -> !{
		ARCH.set_next_timer_intr(TICK_MS);
		self.tasks.get(next_app).unwrap().app_info().user_time.start();
		// init user stack and sret
		unsafe {
			boot_entry(next_app)
		}
	}

	pub fn run_next_at_trap(&self) -> usize{
		let next_app = self.find_next_ready_and_set_run();
		assert!(self.tasks[next_app].status() == TaskStatus::Running);
		let next_task_context = &self.tasks[next_app];
		let next_flow_context = next_task_context.flow_context.get() as *mut FlowContext;
		let trap_handler = trap_handler_in_trap_stage();

		//set sepc, sscratch(user stack)
		unsafe {
			next_flow_context.as_mut().unwrap()
				.load_others();
		}

		// switch task context
		trap_handler.context = unsafe { NonNull::new_unchecked(next_flow_context) };

		assert!(self.tasks[next_app].status() == TaskStatus::Running);
		next_app
	}

	pub fn exit_cur_and_run_next(&self) {
		let app_id = task_context_in_trap_stage().app_info().cur_app;
		let old_task_block = self.tasks.get(app_id).unwrap();
		assert!(old_task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		old_task_block.app_info().kernel_time.end();
		old_task_block.app_info().end();
		old_task_block.mark_exit();

		let next_app = self.run_next_at_trap();

		let new_task_block = &self.tasks[next_app];
		new_task_block.app_info().user_time.start();
		assert!(new_task_block.status() == TaskStatus::Running);
		info!("Kernel end {} and switch to app {}", app_id, next_app);
	}

	pub fn suspend_cur_and_run_next(&self) {
		let app_id = task_context_in_trap_stage().app_info().cur_app;
		let old_task_block = self.tasks.get(app_id).unwrap();
		assert!(old_task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		old_task_block.app_info().kernel_time.end();
		info!("Release {}", app_id);

		old_task_block.mark_suspend_low();
		let next_app = self.run_next_at_trap();
		if next_app != app_id {
			old_task_block.mark_suspend_high_cas(TaskStatus::Ready(ReadyLevel::Low));
		}

		let new_task_block = &self.tasks[next_app];
		new_task_block.app_info().user_time.start();
		assert!(new_task_block.status() == TaskStatus::Running);
		info!("Kernel suspend {} switch to app {}", app_id, next_app);
	}
}
