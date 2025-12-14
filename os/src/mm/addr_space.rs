use crate::mm::address::VPNRange;
use crate::mm::page_table::PageTableTree;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::mm::address::VirtPageNum;
use crate::mm::frame_allocator::FrameTracker;
use bitflags::bitflags;

bitflags! {
	pub struct MapPermission: u8 {
		const R = 1 << 1;
		const W = 1 << 2;
		const X = 1 << 3;
		const U = 1 << 4;
	}
}

pub struct AddrSpace {
	page_table: PageTableTree,
	vma: Vec<VMArea>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
	Identical,
	Framed,
}

pub struct VMArea {
	vpn_range: VPNRange,
	data_frames: BTreeMap<VirtPageNum, FrameTracker>,
		map_type: MapType,
		map_perm: MapPermission,
}

impl AddrSpace {
	pub fn new_bare() -> Self {
		Self { 
			page_table: PageTableTree::new(), 
			vma: Vec::new(),
		}
	}

}