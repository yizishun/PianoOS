use alloc::vec::Vec;
use alloc::vec;
use bitflags::bitflags;
use log::debug;
use riscv::{asm::sfence_vma, register::satp::{self, Satp}};

use crate::{config::PAGE_SIZE, global::FRAME_ALLOCATOR, mm::{address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}, frame_allocator::{FrameAllocator, FrameTracker}}, println};
use alloc::collections::BTreeMap;

const PAGE_ENTRY_NUMBER: usize = PAGE_SIZE / size_of::<PageTableEntry>(); //it will be 512 entry

//TODO:riscv specific
bitflags! {
	#[derive(PartialEq)]
	pub struct PTEFlags: u8 {
		const V = 1 << 0;
		const R = 1 << 1;
		const W = 1 << 2;
		const X = 1 << 3;
		const U = 1 << 4;
		const G = 1 << 5;
		const A = 1 << 6;
		const D = 1 << 7;
	}
}

// PageTableTree
// Record the root PageTableNode location
pub struct PageTableTree {
	pub root_ppn: PhysPageNum,
	//for RAII
	frame_nodes: Vec<FrameTracker>,
	data_frames: BTreeMap<VirtPageNum, FrameTracker>,
}


// PageTableNode
// it is a physical frame, and can be index by ppn
#[repr(C)]
#[repr(align(4096))]
pub struct PageTableNode(pub [PageTableEntry; PAGE_ENTRY_NUMBER]);


#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
	bits: usize
}

impl PageTableTree {
	pub fn new() -> Self {
		let root_ppn = FRAME_ALLOCATOR.get().unwrap().frame_alloc().unwrap();
		Self {
			root_ppn: root_ppn.ppn,
			frame_nodes: vec![root_ppn],
			data_frames: BTreeMap::new(),
		}
	}

	pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags, frames: Option<FrameTracker>) {
		let pte = self.find_pte_create(vpn);
		assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn.0);
		*pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
		if let Some(frame) = frames {
			self.data_frames.insert(vpn, frame);
		}
	}

	pub fn unmap(&mut self, vpn: VirtPageNum) {
		let pte = self.find_pte(vpn).expect("vpn has not be mapped bufore");
		assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn.0);
		*pte = PageTableEntry::EMPTY;
		self.data_frames.remove_entry(&vpn);
	}

	//TODO: arch satp format
	pub fn token(&self) -> usize {
 	       8usize << 60 | self.root_ppn.0
    	}

	pub fn activate_token(&self) {
		unsafe {
			satp::set(satp::Mode::Sv39, 0, self.root_ppn.0);
			sfence_vma(0, 0);
		}

	}

	pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry>{
		Self::walk(self.root_ppn, vpn, |_e: &mut PageTableEntry| {}).cloned()
	}

	// only be called in framed area
	pub fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
		if let Some(frame) = self.data_frames.get(&vpn) {
			Some(frame.ppn)
		} else {
			None
		}
	}

	// only be called in framed area
	pub fn translate_vaddr(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
		let vpn: VirtPageNum = vaddr.vpn_floor();
		if let Some(ppn) = self.translate_vpn(vpn) {
			let base: VirtAddr = vpn.into();
			let offset = vaddr.0 - base.0;
			let paddr: PhysAddr = ppn.into();
			Some((paddr.0 + offset).into())
		} else {
			None
		}
	}

	//helper function
	// walk in this tree and return the entry point to pframe
	fn walk<F>(root_ppn: PhysPageNum, vpn: VirtPageNum, mut on_missing: F) -> Option<&'static mut PageTableEntry>
	where
		F: FnMut(&mut PageTableEntry)
	{
		let vpn_idxs = vpn.indexes();
		let mut node = root_ppn;

		for &idx in &vpn_idxs[0..2] {
			//SAFETY: need to guarantee mut PageTableTree only to access in one hart per time
			let e = unsafe {
				node.get_pte_node().entry_mut(idx)
			};

			// when missing
			if !e.is_valid() {
				on_missing(e);
				if !e.is_valid() {
					return None
				}
			}

			// walk
			node = e.ppn();
		}
		Some(unsafe {
			node.get_pte_node().entry_mut(vpn_idxs[2])
		})
	}

	// TODO: if frame alloc false, code will unwrap(then panic)
	fn find_pte_create(&mut self, vpn: VirtPageNum) -> &mut PageTableEntry {
		let on_missing = |e: &mut PageTableEntry| {
			let frame = FRAME_ALLOCATOR.get().unwrap().frame_alloc().unwrap();
			*e = PageTableEntry::new(frame.ppn, PTEFlags::V);
			self.frame_nodes.push(frame);
		};
		Self::walk(self.root_ppn, vpn, on_missing).unwrap()
	}

	fn find_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
		let on_missing = |_e: &mut PageTableEntry| {};
		Self::walk(self.root_ppn, vpn, on_missing)
	}
}

impl PageTableNode {
	pub fn from_ppn(ppn: PhysPageNum) -> &'static mut Self {
		let pa: PhysAddr = ppn.into();
		let ptr = (pa.0 as usize) as *mut Self;
		unsafe {
			&mut *ptr
		}
	}

	pub fn entry(&self, index: usize) -> &PageTableEntry {
		&self.0[index]
	}

	pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
		&mut self.0[index]
	}
}

//TODO:riscv specific
impl PageTableEntry {
	pub const EMPTY: PageTableEntry =
		PageTableEntry {
			bits: 0
		};

	pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
		PageTableEntry {
			bits: ppn.0 << 10 | flags.bits() as usize
		}
	}

	pub fn ppn(&self) -> PhysPageNum {
        	(self.bits >> 10 & ((1usize << 44) - 1)).into()
    	}

	pub fn flags(&self) -> PTEFlags {
		PTEFlags::from_bits(self.bits as u8).unwrap()
	}

	pub fn is_valid(&self) -> bool {
		(self.flags() & PTEFlags::V) != PTEFlags::empty()
	}

	pub fn readable(&self) -> bool {
		(self.flags() & PTEFlags::R) != PTEFlags::empty()
	}

	pub fn writable(&self) -> bool {
		(self.flags() & PTEFlags::W) != PTEFlags::empty()
	}

	pub fn executable(&self) -> bool {
		(self.flags() & PTEFlags::X) != PTEFlags::empty()
	}

}
