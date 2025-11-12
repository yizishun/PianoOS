use core::ops::Range;
use core::ptr::{NonNull, null};
use core::sync::atomic::Ordering;
use core::array;

use log::info;
use spin::mutex::Mutex;

use crate::global::{ARCH, KERNEL_STACK, LOADER};
use crate::arch::common::{ArchPower, FlowContext, boot_entry, boot_handler};
use crate::config::MAX_APP_NUM;
use crate::harts::{task_context_in_trap_stage, trap_handler_in_trap_stage};
use crate::task::block::TaskControlBlock;
use crate::task::status::TaskStatus;

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
				TaskControlBlock::new(i, app_range[i].start as usize)
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
		// init task context
		unsafe {
			next_task_context.app_info.get().as_mut().unwrap()
				.start(next_app, next_app_range.clone());
		}
		// init sepc, sstatus, stvec
		boot_handler(next_app_range.start as usize);
		next_app
	}

	pub fn run_next_at_boot(&self, next_app: usize) -> !{
		// init user stack and sret
		unsafe {
			boot_entry(next_app)
		}
	}

	pub fn run_next_at_trap(&self) {
		let next_app = self.find_next_ready_and_set_run();
		let next_task_context = &self.tasks[next_app];
		let next_flow_context = next_task_context.flow_context.get() as *mut FlowContext;
		let next_app_range = &self.app_range[next_app];
		let trap_handler = trap_handler_in_trap_stage();

		//set sepc, sscratch(user stack)
		unsafe {
			next_flow_context.as_mut().unwrap()
				.load_others();
		}

		// switch task context
		trap_handler.context = unsafe { NonNull::new_unchecked(next_flow_context) };

		// init task context
		unsafe {
			next_task_context.app_info.get().as_mut().unwrap()
				.start(next_app, next_app_range.clone());
		}

		info!("Kernel loading app({})", next_app);
	}

	pub fn exit_cur_and_run_next(&self) {
		let app_id = unsafe { task_context_in_trap_stage().app_info.get().as_ref().unwrap().cur_app };
		let task_block = self.tasks.get(app_id).unwrap();
		assert!(task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		task_block.mark_exit();
		self.run_next_at_trap();
	}

	pub fn suspend_cur_and_run_next(&self) {
		let app_id = unsafe { task_context_in_trap_stage().app_info.get().as_ref().unwrap().cur_app };
		let task_block = self.tasks.get(app_id).unwrap();
		assert!(task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		task_block.mark_suspend();
		self.run_next_at_trap();
	}
}
