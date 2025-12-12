use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::{config, global::{self, FRAME_ALLOCATOR}, mm::address::PhysPageNum};

pub trait FrameAllocatorInterface: Send {
	fn alloc(&mut self) -> Option<PhysPageNum>;
	fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct FrameAllocator {
	inner: Mutex<Box<dyn FrameAllocatorInterface>> //to support change in runtime
}

// when alloc a Frame, bind it to a Tracker
// it is RAII
pub struct FrameTracker {
	pub ppn: PhysPageNum
}

impl FrameTracker {
	pub fn new(ppn: PhysPageNum) -> Self {
		// page cleaning
		let byte_array = ppn.get_byte_array();
		byte_array.iter_mut().for_each(|b| *b = 0);

		Self { ppn }
	}
}

impl Drop for FrameTracker {
	fn drop(&mut self) {
		FRAME_ALLOCATOR.get().unwrap().frame_dealloc(self.ppn);
	}
}

impl FrameAllocator {
	pub fn new(inner: Box<dyn FrameAllocatorInterface>) -> Self {
		Self { 
			inner: Mutex::new(inner)
		}
	}

	pub fn frame_alloc(&self) -> Option<FrameTracker> {
		if let Some(ppn) = self.inner.lock().alloc() {
			Some(FrameTracker::new(ppn))
		} else {
			None
		}
	}

	pub fn frame_dealloc(&self, ppn: PhysPageNum) {
		self.inner.lock().dealloc(ppn);
	}
}

pub struct StackFrameAllocator {
	current: usize,
	end: usize,
	recycled: Vec<usize>
}

impl FrameAllocatorInterface for StackFrameAllocator {
	fn alloc(&mut self) -> Option<PhysPageNum> {
		assert!(self.current <= self.end);
		if let Some(ppn) = self.recycled.pop() {
			Some(ppn.into())
		} else {
			if self.current == self.end {
				None
			} else {
				self.current += 1;
				Some((self.current - 1).into())
			}
		}
	}
	
	fn dealloc(&mut self, ppn: PhysPageNum) {
		assert!(self.current <= self.end);
		let ppn = ppn.0;
		// validity check
		if ppn >= self.current || 
		   self.recycled
			.iter()
			.find(|&v| {*v == ppn})
			.is_some() {
			panic!("Frame ppn={:#x} has not been allocated!", ppn);
		}	
		// recycle
        	self.recycled.push(ppn);
	}
}

impl StackFrameAllocator {
	pub fn new() -> Self {
		Self {
			current: 0,
			end: 0,
			recycled: Vec::new(),
		}
	}

	pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
		self.current = l.0;
		self.end = r.0;
	}

	pub fn init_scope(&mut self) {
		let l = PhysPageNum::from(unsafe{ global::ekernel });
		let r = PhysPageNum::from(config::MEMORY_END);
		self.init(l, r);
	}
}