use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
use core::cell::SyncUnsafeCell;
use crate::arch::common::FlowContext;
use crate::task::status::TaskStatus;
use crate::task::harts::AppHartInfo;

//TODO: 最好做成可分配可回收的结构
pub struct TaskControlBlock {
	// SAFETY: one flow_context will only bind to one harts
	pub flow_context: SyncUnsafeCell<FlowContext>,
	pub task_status: AtomicU8,
	pub app_info: SyncUnsafeCell<AppHartInfo>
}

impl TaskControlBlock {
	pub fn new(app_id: usize, start_addr: usize) -> Self {
		let task_status = AtomicU8::new(TaskStatus::Ready as u8);
		let app_info = SyncUnsafeCell::new(AppHartInfo::new(app_id));
		let flow_context= SyncUnsafeCell::new(FlowContext::new(app_id, start_addr));
		Self {
			flow_context,
			task_status, 
			app_info
		}
	}

	pub fn status(&self) -> TaskStatus {
		TaskStatus::try_from(self.task_status.load(Ordering::Relaxed))
			.unwrap()
	}

	pub fn mark_suspend(&self) {
		self.task_status.store(TaskStatus::Ready as u8, Ordering::Release);
	}

	pub fn mark_exit(&self) {
		self.task_status.store(TaskStatus::Exited as u8, Ordering::Release);
	}

	pub fn mark_runing(&self) {
		self.task_status.store(TaskStatus::Running as u8, Ordering::Release);
	}

	pub fn mark_uninit(&self) {
		self.task_status.store(TaskStatus::UnInit as u8, Ordering::Release);
	}
}
