use alloc::{boxed::Box, vec::Vec};
use log::{debug, info};
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt;
use spin::Mutex;

use crate::mm::address::PhysAddr;
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

impl Debug for FrameTracker {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
	}
}

impl FrameTracker {
	pub fn new(ppn: PhysPageNum) -> Self {
		// page cleaning
		// SAFETY: it the first time get it byte array
		let byte_array = unsafe { ppn.get_byte_array() };
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
		let laddr = PhysAddr::from(&raw const global::ekernel as usize);
		let raddr = PhysAddr::from(config::MEMORY_END);
		let lppn = laddr.ppn_ceil();
		let rppn = raddr.ppn_floor();
		info!("Frame start addr: {:#x}, start ppn: {:#x}", laddr.0, lppn.0);
		info!("Frame end   addr: {:#x}, end ppn: {:#x}", raddr.0, rppn.0);
		self.init(lppn, rppn);
	}
}

#[allow(unused)]
pub fn frame_allocator_test() {
	let mut v: Vec<FrameTracker> = Vec::new();
	for i in 0..5 {
		let frame = FRAME_ALLOCATOR.get().unwrap().frame_alloc().unwrap();
		debug!("{:?}", frame);
		v.push(frame);
	}
	v.clear();
	for i in 0..5 {
		let frame = FRAME_ALLOCATOR.get().unwrap().frame_alloc().unwrap();
		debug!("{:?}", frame);
		v.push(frame);
	}
	drop(v);
	debug!("frame_allocator_test passed!");
}