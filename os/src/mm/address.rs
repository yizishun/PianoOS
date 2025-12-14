use core::slice::from_raw_parts_mut;

use log::info;

use crate::{config::{self, PAGE_SIZE, PAGE_SIZE_BITS}, mm::page_table::PageTableNode};

const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

// RISC-V SV39 Physical Address(Total 39 bits)
// +-------------------------+------------+------------+----------------------+
// |          PPN[2]         |   PPN[1]   |   PPN[0]   |      Page Offset     |
// |         26 bits         |   9 bits   |   9 bits   |        12 bits       |
// |         [55..30]        |  [29..21]  |  [20..12]  |        [11..0]       |
// +-------------------------+------------+------------+----------------------+
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

// RISC-V SV39 Virtual Address(Total 56 bits)
// +------------+------------+------------+----------------------+
// |   VPN[2]   |   VPN[1]   |   VPN[0]   |      Page Offset     |
// |   9 bits   |   9 bits   |   9 bits   |        12 bits       |
// |  [38..30]  |  [29..21]  |  [20..12]  |        [11..0]       |
// +------------+------------+------------+----------------------+
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
	pub unsafe fn get_byte_array(&self) -> &'static mut [u8] {
		let pa:PhysAddr = (*self).into();
		unsafe {
			from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE)
		}
	}

	pub unsafe fn get_pte_node(&self) -> &'static mut PageTableNode {
		PageTableNode::from_ppn(*self)
	}

	pub unsafe fn get_mut<T>(&self) -> &'static mut T {
		let pa: PhysAddr = self.clone().into();
		unsafe {
			(pa.0 as *mut T).as_mut().unwrap()
		}
    }
}

impl VirtPageNum {
	pub fn indexes(&self) -> [usize; 3] {
		let mut vpn = self.0;
		let mut idx = [0usize; 3];
		for i in (0..3).rev() {
			idx[i] = vpn & 511;
			vpn >>= 9;
		}
		idx
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
