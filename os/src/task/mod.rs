use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use core::intrinsics::forget;
use core::array;

use log::info;
use spin::mutex::Mutex;

use crate::global::{ARCH, ELFS_INFO, KERNEL_STACK, TASK_MANAGER};
use crate::arch::common::{Arch, ArchPower, ArchTime, ArchTrap, FlowContext};
use crate::config::{FLOW_CONTEXT_VADDR, MAX_APP_NUM, TICK_MS, TRAMPOLINE_VADDR, TRAP_HANDLER_VADDR};
use crate::harts::{task_context_in_trap_stage, trap_handler_in_trap_stage};
use crate::task::block::TaskControlBlock;
use crate::task::status::{ReadyLevel, TaskStatus};

pub mod harts;
pub mod block;
pub mod status;

pub struct TaskManager {
	pub num_app: usize,
	finished: Mutex<bool>,
	tasks: [TaskControlBlock; MAX_APP_NUM]
}

unsafe impl Send for TaskManager {}
unsafe impl Sync for TaskManager {}

impl TaskManager {
	pub fn new() -> Self {
		let num_app = ELFS_INFO.get().unwrap().num_app;
		let tasks: [TaskControlBlock; MAX_APP_NUM] =
			array::from_fn(|i| {
				if i < num_app {
					TaskControlBlock::new(
						i,
						TaskStatus::Ready(ReadyLevel::High),
						Some(ELFS_INFO.get().unwrap().elf_info(i))
					)
				} else {
					TaskControlBlock::new(
						i,
						TaskStatus::Exited,
						None
					)
				}
			});
		TaskManager {
			num_app: num_app,
			finished: Mutex::new(false),
			tasks
		}
	}

	pub fn map_flow_context(&self) {
		self.tasks.iter().for_each(|tcb| {
			// self ref
			tcb.addr_space().insert_uflow_context(
				(&tcb.flow_context as *const _ as usize).into()
			);
		});
	}

	pub fn app_size(&self, app_id: usize) -> usize {
		self.tasks[app_id].base_size
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
		let next_tcb = &self.tasks[next_app];
		let next_flow_context = unsafe {
			NonNull::new_unchecked((&self.tasks[next_app]).flow_context.get() as *mut _)
		};
		//TODO: use next_flow_context translated result
		let next_flow_context_va = unsafe {
			NonNull::new_unchecked(FLOW_CONTEXT_VADDR as *mut _)
		};

		// link kernel stack to user app
		let kstack = unsafe {
			#[allow(static_mut_refs)]
			KERNEL_STACK.get_mut(hartid).unwrap()
				.init_trap_stack(
					hartid,
					next_flow_context_va,
					<Arch as ArchTrap>::fast_handler_user,
					|_| {}
				)
		};

		// link user app to kernel stack(traph)
		//map traph
		next_tcb.addr_space().insert_utrap_handler((*&kstack).kstack_ptr().into());
		// init sepc, sstatus, stvec, stie, sscratch
		<Arch as ArchTrap>::boot_handler(
			next_tcb.flow_context().pc,
			TRAMPOLINE_VADDR,
			TRAP_HANDLER_VADDR, //TODO: use kstack translated result
		);
		forget(kstack);
		next_app
	}

	pub fn run_next_at_boot(&self, next_app: usize) -> !{
		ARCH.set_next_timer_intr(TICK_MS);
		self.tasks.get(next_app).unwrap().app_info().user_time.start();
		let sp = TASK_MANAGER.get().unwrap().tasks[next_app].flow_context().sp;
		let addr_space = TASK_MANAGER.get().unwrap().tasks[next_app].addr_space().token();
		// init user stack and sret
		unsafe {
			<Arch as ArchTrap>::boot_entry(sp, addr_space)
		}
	}

	pub fn run_next_at_trap(&self) -> usize{
		let next_app = self.find_next_ready_and_set_run();
		assert!(self.tasks[next_app].status() == TaskStatus::Running);
		let next_flow_context = (&self.tasks[next_app]).flow_context.get() as *mut FlowContext;
		let trap_handler = trap_handler_in_trap_stage();

		// switch task context
		trap_handler.context = unsafe { NonNull::new_unchecked(next_flow_context) };

		assert!(self.tasks[next_app].status() == TaskStatus::Running);
		next_app
	}

	pub fn exit_cur_and_run_next(&self) {
		let app_id = task_context_in_trap_stage().app_info().app_id;
		let old_task_block = self.tasks.get(app_id).unwrap();
		assert!(old_task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		old_task_block.app_info().kernel_time.end();
		old_task_block.app_info().end();
		old_task_block.mark_exit();

		let next_app = self.run_next_at_trap();

		let new_task_block = &self.tasks[next_app];
		assert!(new_task_block.status() == TaskStatus::Running);
		info!("Kernel end {} and switch to app {}", app_id, next_app);
	}

	pub fn suspend_cur_and_run_next(&self) {
		let app_id = task_context_in_trap_stage().app_info().app_id;
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
		assert!(new_task_block.status() == TaskStatus::Running);
		info!("Kernel suspend {} switch to app {}", app_id, next_app);
	}
}
