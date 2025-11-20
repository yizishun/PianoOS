use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
use core::cell::SyncUnsafeCell;
use crate::arch::common::FlowContext;
use crate::task::status::ReadyLevel;
use crate::task::status::TaskStatus;
use crate::task::harts::AppHartInfo;

//TODO: 最好做成可分配可回收的结构
pub struct TaskControlBlock {
	// SAFETY: one flow_context will only bind to one harts
	pub flow_context: SyncUnsafeCell<FlowContext>,
	pub task_status: AtomicU8,
	app_info: SyncUnsafeCell<AppHartInfo>
}

impl TaskControlBlock {
	pub fn new(app_id: usize, start_addr: usize, end_addr: usize, status: TaskStatus) -> Self {
		let task_status = AtomicU8::new(u8::from(status));
		let app_info = SyncUnsafeCell::new(AppHartInfo::new(app_id, start_addr, end_addr));
		let flow_context= SyncUnsafeCell::new(FlowContext::new(app_id, start_addr));
		Self {
			flow_context,
			task_status, 
			app_info
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
