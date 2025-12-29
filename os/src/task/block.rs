use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
use core::cell::SyncUnsafeCell;
use log::debug;

use crate::arch::common::FlowContext;
use crate::config::TRAP_HANDLER_VADDR;
use crate::global::TASK_MANAGER;
use crate::mm::addr_space::AddrSpace;
use crate::task::status::ReadyLevel;
use crate::task::status::TaskStatus;
use crate::task::harts::AppHartInfo;

//TODO: 最好做成可分配可回收的结构
#[repr(C, align(4096))]
pub struct TaskControlBlock {
	// SAFETY: one flow_context will only bind to one harts
	pub flow_context: SyncUnsafeCell<FlowContext>,
	pub task_status: AtomicU8,
	app_info: SyncUnsafeCell<AppHartInfo>,
	pub addr_space: SyncUnsafeCell<AddrSpace>,
	pub base_size: usize
}

impl TaskControlBlock {
	pub fn new(
		app_id: usize,
		status: TaskStatus,
		elf_data: Option<&[u8]>,
	) -> Self {
		let task_status = AtomicU8::new(u8::from(status));
		if elf_data == None {
			return Self {
				flow_context: SyncUnsafeCell::new(FlowContext::ZERO),
				task_status,
				app_info: SyncUnsafeCell::new(AppHartInfo::ZERO),
				addr_space: SyncUnsafeCell::new(AddrSpace::new_bare()),
				base_size: 0
			};
		} else {
			let (u_addr_space, u_sp, u_entry) = AddrSpace::from_elf(elf_data.unwrap());
			let app_info = SyncUnsafeCell::new(AppHartInfo::new(app_id, elf_data.unwrap().as_ptr_range()));
			let flow_context= SyncUnsafeCell::new(FlowContext::new(
				u_sp,
				u_entry,
				app_id,
				u_addr_space.token(),
				0)); //utrah will be set in link with hart place
			Self {
				flow_context, //lack utraph
				task_status,
				app_info,
				addr_space: SyncUnsafeCell::new(u_addr_space), //lack the map of utraph and uflow
				base_size: u_sp
			}
		}
	}

	pub fn app_info(&self) -> &mut AppHartInfo {
		unsafe {
			&mut (*self.app_info.get())
		}
	}

	pub fn flow_context(&self) -> &mut FlowContext {
		unsafe {
			&mut (*self.flow_context.get())
		}
	}

	pub fn status(&self) -> TaskStatus {
		TaskStatus::try_from(self.task_status.load(Ordering::SeqCst))
			.unwrap()
	}

	pub fn addr_space(&self) -> &mut AddrSpace {
		unsafe {
			&mut (*self.addr_space.get())
		}
	}

	pub fn mark_suspend_low(&self) {
		self.task_status.store(u8::from(TaskStatus::Ready(ReadyLevel::Low)), Ordering::Release);
	}

	pub fn mark_suspend_high_cas(&self, cur: TaskStatus) {
		let _ = self.task_status.compare_exchange(
			u8::from(cur),
			u8::from(TaskStatus::Ready(ReadyLevel::High)),
			Ordering::Acquire,
			Ordering::Relaxed
		);
	}

	pub fn mark_exit(&self) {
		self.task_status.store(u8::from(TaskStatus::Exited), Ordering::Release);
	}

	pub fn mark_runing(&self) {
		self.task_status.store(u8::from(TaskStatus::Running), Ordering::Release);
	}

	pub fn mark_uninit(&self) {
		self.task_status.store(u8::from(TaskStatus::UnInit), Ordering::Release);
	}
}
