use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
use crate::task::{context::TaskContext, status::TaskStatus};

pub struct TaskControlBlock {
	pub task_status: AtomicU8,
	pub task_cx: TaskContext
}

impl TaskControlBlock {
	pub fn new(kstack_ptr: usize) -> Self {
		let task_status = AtomicU8::new(TaskStatus::Ready as u8);
		let task_cx = TaskContext::goto_restore(kstack_ptr);
		Self { task_status, task_cx }
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
