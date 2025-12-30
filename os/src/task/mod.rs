use core::cmp::Reverse;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;
use core::intrinsics::forget;
use core::array;

use log::info;
use spin::mutex::Mutex;

use crate::arch::loongarch64::trap;
use crate::global::{ARCH, ELFS_INFO, KERNEL_ADDRSPACE, KERNEL_STACK, TASK_MANAGER};
use crate::arch::common::{Arch, ArchPower, ArchTime, ArchTrap, FlowContext};
use crate::config::{FLOW_CONTEXT_VADDR, HART_CONTEXT_VADDR, MAX_APP_NUM, PAGE_SIZE, PAGE_SIZE_BITS, TICK_MS, TRAMPOLINE_VADDR, TRAP_HANDLER_VADDR};
use crate::harts::{HartContext, task_context_in_trap_stage, trap_handler_in_trap_stage};
use crate::mm::stack::KernelStack;
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
						TaskStatus::UnInit,
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
	/// return (prev_status, app_id)
	/// TODO: use bitmap or queue(linux kernel does) to opt it
	fn find_next_ready_and_set_run(&self) -> (TaskStatus, usize) {
		loop {
			let best_candidate = self.tasks.iter().enumerate()
				.filter_map(|(id, task)| {
					let raw = task.task_status.load(Ordering::Relaxed);
					let status = TaskStatus::try_from(raw).ok()?;
					let prio = status.get_priority()?;
					Some((id, raw, status, prio))
				}).max_by_key(|(id, _, _, prio)| (*prio, Reverse(*id)));

			if let Some((id, raw, status, _)) = best_candidate {
				if self.tasks[id].task_status.compare_exchange(
					raw,
					u8::from(TaskStatus::Running),
					Ordering::Acquire,
					Ordering::Relaxed).is_ok() {
						return (status, id);
					}
				else {
				    self.check_end();
				}
			}
		}
	}


	pub fn prepare_next_at_boot(&self, hartid: usize) -> usize {
		let (prev_status, next_app) = self.find_next_ready_and_set_run();
		let next_tcb = &self.tasks[next_app];
		//TODO: use next_flow_context translated result
		let next_flow_context_va = unsafe {
			NonNull::new_unchecked(FLOW_CONTEXT_VADDR as *mut _)
		};
		let hart_context_va = unsafe {
			NonNull::new_unchecked(HART_CONTEXT_VADDR as *mut _)
		};

		// kernel: link kernel stack to user app
		let mut hart_context = HartContext::new();
		#[allow(static_mut_refs)]
		hart_context.init(
			hartid,
			KERNEL_ADDRSPACE.get().unwrap().token(),
			unsafe { KERNEL_STACK.get(hartid).unwrap().traph() as *const _ as usize }
		);
		let kstack = unsafe {
			#[allow(static_mut_refs)]
			KERNEL_STACK.get_mut(hartid).unwrap()
				.init_trap_stack(
					next_flow_context_va,
					hart_context_va,
					hart_context,
					<Arch as ArchTrap>::fast_handler_user,
					|_| {}
				)
		};

		// user: link user app to kernel stack(traph), i.e. map some kernel staff
		//map traph
		assert!(prev_status == TaskStatus::UnInit);
		next_tcb
			.addr_space()
			.insert_utrap_handler((*&kstack).kstack_ptr().into(), false);
		//map kernel context(hart context)
		#[allow(static_mut_refs)]
		next_tcb
			.addr_space()
			.insert_uhart_context((unsafe{
				KERNEL_STACK.get_mut(hartid).unwrap().as_ptr_range().start as usize
			}).into(), false);
		// traph not align to 4k, so we should find the offset of traph
		let offset = (*&kstack).kstack_ptr() & (PAGE_SIZE - 1);
		next_tcb.flow_context().utrap_handler = TRAP_HANDLER_VADDR + offset;

		// init sepc, sstatus, stvec, stie, sscratch
		<Arch as ArchTrap>::boot_handler(
			next_tcb.flow_context().pc,
			TRAMPOLINE_VADDR,
			TRAP_HANDLER_VADDR + offset,
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
		let (prev_status, next_app) = self.find_next_ready_and_set_run();
		assert!(self.tasks[next_app].status() == TaskStatus::Running);
		let next_tcb = &self.tasks[next_app];
		let next_flow_context = (&self.tasks[next_app]).flow_context.get() as *mut FlowContext;
		let trap_handler = trap_handler_in_trap_stage();
		let hartid = trap_handler.hart_id;

		// kernel: switch task context
		trap_handler.transed_context = unsafe { NonNull::new_unchecked(next_flow_context) };
		trap_handler.app_id = next_app;

		// user: modify the map and flow_context
		assert!(matches!(prev_status, TaskStatus::UnInit | TaskStatus::Ready(_)));
		let is_uninit = prev_status == TaskStatus::UnInit;
		//map traph
		next_tcb
			.addr_space()
			.insert_utrap_handler((trap_handler as *const _ as usize).into(), !is_uninit);
		//map kernel context(hart context)
		#[allow(static_mut_refs)]
		next_tcb
			.addr_space()
			.insert_uhart_context((unsafe{
				KERNEL_STACK.get_mut(hartid).unwrap().as_ptr_range().start as usize
			}).into(), !is_uninit);
		// offset and utraph va is always the same
		let offset = (trap_handler as *const _ as usize) & (PAGE_SIZE - 1);
		if is_uninit {
			next_tcb.flow_context().utrap_handler = TRAP_HANDLER_VADDR + offset;
		}

		// switch sscratch and sepc
		unsafe {
			(*next_flow_context).load_others();
		}

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

	pub fn suspend_cur_and_run_next(&self, sp: Option<usize>, pc: Option<usize>) {
		let app_id = task_context_in_trap_stage().app_info().app_id;
		let old_task_block = self.tasks.get(app_id).unwrap();
		assert!(old_task_block.status() == TaskStatus::Running, "this task is not Running, something may be wrong");
		old_task_block.app_info().kernel_time.end();

		info!("Release {}, and save the sp and pc", app_id);
		if let Some(sp) = sp {
			old_task_block.flow_context().set_sp(sp);
		}
		if let Some(pc) = pc {
			old_task_block.flow_context().set_pc(pc);
		}

		old_task_block.mark_suspend_low();
		let next_app = self.run_next_at_trap();
		if next_app != app_id {
			old_task_block.mark_suspend_high_cas(TaskStatus::Ready(ReadyLevel::Low));
		}

		let new_task_block = &self.tasks[next_app];
		assert!(new_task_block.status() == TaskStatus::Running);
		info!("Kernel suspend {} switch to app {}", app_id, next_app);
	}

	pub fn task(&self, app_id: usize) -> &TaskControlBlock {
		// check the app_id?
		&self.tasks[app_id]
	}
}
