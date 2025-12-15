use core::ops::Range;

use crate::config::{MEMORY_END, PAGE_SIZE};
use crate::global::FRAME_ALLOCATOR;
use crate::mm::address::{PhysPageNum, VPNRange, VirtAddr};
use crate::mm::page_table::{self, PTEFlags, PageTableTree};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::mm::address::VirtPageNum;
use crate::mm::frame_allocator::FrameTracker;
use bitflags::bitflags;
use log::info;
use crate::global::*;

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

pub(crate) struct VMArea {
	vpn_range: VPNRange,
	data_frames: BTreeMap<VirtPageNum, FrameTracker>,
	map_type: MapType,
	map_perm: MapPermission,
}

impl AddrSpace {
	/// Create an empty address space
	pub fn new_bare() -> Self {
		Self { 
			page_table: PageTableTree::new(), 
			vma: Vec::new(),
		}
	}

	/// Push a VMArea into the address space and optionally copy data
	fn push(&mut self, mut vma: VMArea, data: Option<&[u8]>){
		(&mut vma).map_all(&mut self.page_table);
		if let Some(data) = data { 
			vma.copy_data(data) 
		}
		self.vma.push(vma);
	}

	/// Create and insert a framed VMArea with given range and permissions
	pub fn insert_framed_area(&mut self, 
		start_va: VirtAddr, 
		end_va: VirtAddr, 
		perm: MapPermission) {
		let vma = VMArea::new(start_va, end_va, MapType::Framed, perm);
		self.push(vma, None);
	}

	/// Create the kernel address space
	pub fn new_kernel() -> Self {
		let mut kernel_space = Self::new_bare();
		print_kernel_mem();
		// trampoline
		kernel_space.map_trampoline();

		// .text
		kernel_space.push(VMArea::new(
			(&raw const stext as usize).into(),
			(&raw const etext as usize).into(),
			MapType::Identical,
			MapPermission::R | MapPermission::X,
		), None);

		// .rodata
		kernel_space.push(VMArea::new(
			(&raw const srodata as usize).into(),
			(&raw const erodata as usize).into(),
			MapType::Identical,
			MapPermission::R,
		), None);

		// .data
		kernel_space.push(VMArea::new(
			(&raw const sdata as usize).into(),
			(&raw const edata as usize).into(),
			MapType::Identical,
			MapPermission::R | MapPermission::W,
		), None);

		// .bss
		kernel_space.push(VMArea::new(
			(&raw const sbss as usize).into(),
			(&raw const ebss as usize).into(),
			MapType::Identical,
			MapPermission::R | MapPermission::W,
		), None);

		// remain space
		kernel_space.push(VMArea::new(
			(&raw const ekernel as usize).into(),
			(MEMORY_END as usize).into(),
			MapType::Identical,
			MapPermission::R | MapPermission::W,
		), None);

		info!("kernel mapping address space build complete");
		kernel_space
	}

	pub fn map_trampoline(&mut self) {
		todo!()
	}

	/// Create address space from ELF, returning (self, user_sp, entry_point)
	pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
		todo!()
	}

}

impl VMArea {
	/// Create a new VMArea with specified range, type and permissions
	pub fn new(
		start_va: VirtAddr,
		end_va: VirtAddr,
		map_type: MapType,
		map_perm: MapPermission
	) -> Self {
		let start_vpn = start_va.vpn_floor();
		let end_vpn = end_va.vpn_ceil();
		Self { 
			vpn_range: (start_vpn..end_vpn), 
			data_frames: BTreeMap::new(),
			map_type,
			map_perm,
		}
	}

	/// Map all pages in the VMArea to the page table
	pub fn map_all(&mut self, pt_tree: &mut PageTableTree) {
		for vpn in self.vpn_range.clone() {
			self.map_one(pt_tree, vpn);
		}
	}

	/// Unmap all pages in the VMArea from the page table
	pub fn unmap_all(&mut self, pt_tree: &mut PageTableTree) {
		for vpn in self.vpn_range.clone() {
			self.unmap_one(pt_tree, vpn);
		}
	}

	/// Copy data into the VMArea's frames
	pub fn copy_data(&mut self, data: &[u8]) {
		assert_eq!(self.map_type, MapType::Framed); // identical map can be directly access by vpn
		let mut cur_start: usize;
		let len = data.len();

		for (vpn, data_chunk) in self.vpn_range.clone().zip(data.chunks(PAGE_SIZE)) {
			let ppn = self.data_frames.get(&vpn).unwrap().ppn; //TODO: 需不需要对缺页的情况进行检查

			let dst = unsafe { ppn.get_byte_array() }; //SAFETY: this area will only be access by one cpu in one task
			let src = data_chunk;
			
			dst[..src.len()].copy_from_slice(src);
		}

	}

	/// Map a single page in the VMArea
	fn map_one(&mut self, pt_tree: &mut PageTableTree, vpn: VirtPageNum) {
		assert!(self.vpn_range.contains(&vpn));
		let ppn: PhysPageNum = match self.map_type {
			MapType::Identical => {PhysPageNum(vpn.0)},
			MapType::Framed => {
				let frame = FRAME_ALLOCATOR.get().unwrap().frame_alloc().unwrap();
				let ppn = frame.ppn;
				self.data_frames.insert(vpn, frame);
				ppn
			}
		};
		pt_tree.map(vpn, ppn,
			PTEFlags::from_bits(self.map_perm.bits()).unwrap());
	}

	/// Unmap a single page in the VMArea
	fn unmap_one(&mut self, pt_tree: &mut PageTableTree, vpn: VirtPageNum) {
		match self.map_type {
			MapType::Framed => 
				{ self.data_frames
					.remove_entry(&vpn)
					.expect("vpn has not mapped before");},
			MapType::Identical => {},
		}
		pt_tree.unmap(vpn);
	}
}

pub fn print_kernel_mem() {
	unsafe {
		info!("kernel memory map:");
		info!("kernel base = {:<10p}", &skernel);
		info!(".text      : [{:<10p}, {:<10p}]", &stext, &etext);
		info!(".rodata    : [{:<10p}, {:<10p}]", &srodata, &erodata);
		info!(".data      : [{:<10p}, {:<10p}]", &sdata, &edata);
		info!(".bss.kstack: [{:<10p}, {:<10p}]", &skstack, &ekstack);
		info!(".bss.ustack: [{:<10p}, {:<10p}]", &sustack, &eustack);
		info!(".bss.heap  : [{:<10p}, {:<10p}]", &sheap, &eheap);
		info!(".bss       : [{:<10p}, {:<10p}]", &sbss, &ebss);
		info!("kernel end = {:<10p}", &ekernel);
	}
}