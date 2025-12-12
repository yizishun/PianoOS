use core::slice::from_raw_parts_mut;

use crate::config::{self, PAGE_SIZE, PAGE_SIZE_BITS};

const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl From<usize> for PhysAddr {
    	fn from(v: usize) -> Self { Self(v & ( (1 << PA_WIDTH_SV39) - 1 )) }
}
impl From<usize> for PhysPageNum {
    	fn from(v: usize) -> Self { Self(v & ( (1 << PPN_WIDTH_SV39) - 1 )) }
}
impl From<usize> for VirtAddr {
    	fn from(v: usize) -> Self { Self(v & ( (1 << VA_WIDTH_SV39) - 1 )) }
}
impl From<usize> for VirtPageNum {
    	fn from(v: usize) -> Self { Self(v & ( (1 << VPN_WIDTH_SV39) - 1 )) }
}

impl PhysPageNum {
	pub fn get_byte_array(&self) -> &'static mut [u8] {
		let base_addr:PhysAddr = (*self).into();
		unsafe {
			from_raw_parts_mut(base_addr.0 as *mut u8, PAGE_SIZE)
		}
	}
}

impl From<PhysPageNum> for PhysAddr {
	fn from(ppn: PhysPageNum) -> Self {
		Self (ppn.0 << PAGE_SIZE_BITS)
	}
}

impl PhysAddr {
	pub fn page_offset(&self) -> usize {
		self.0 & (PAGE_SIZE - 1)
	}

	pub fn ppn_floor(&self) -> PhysPageNum {
		PhysPageNum(self.0 >> PAGE_SIZE_BITS)
	}

	pub fn ppn_ceil(&self) -> PhysPageNum {
		PhysPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_SIZE_BITS)
	}
}

impl VirtAddr {
	pub fn page_offset(&self) -> usize {
		self.0 & (PAGE_SIZE - 1)
	}

	pub fn vpn_floor(&self) -> VirtPageNum {
		VirtPageNum(self.0 >> PAGE_SIZE_BITS)
	}

	pub fn vpn_ceil(&self) -> VirtPageNum {
		VirtPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_SIZE_BITS)
	}
}
