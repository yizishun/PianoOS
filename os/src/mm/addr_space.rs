use core::fmt::{Display, write};
use core::fmt::Formatter;
use core::ops::Range;
use core::iter::Step;

use crate::config::{APP_VIRT_ADDR, FLOW_CONTEXT_VADDR, HART_CONTEXT_VADDR, MEMORY_END, PAGE_SIZE, TRAMPOLINE_VADDR, TRAP_HANDLER_VADDR, USER_STACK_SIZE};
use crate::global::FRAME_ALLOCATOR;
use crate::mm::address::{PhysAddr, PhysPageNum, VPNRange, VirtAddr};
use crate::mm::page_table::{PTEFlags, PageTableTree};
use alloc::vec::Vec;
use elf::abi::{ET_DYN, PF_R, PF_W, PF_X};
use crate::mm::address::VirtPageNum;
use bitflags::bitflags;
use log::{debug, info};
use crate::{global::*, println};
use elf::{
	ElfBytes,
	abi::{PT_LOAD, R_RISCV_RELATIVE},
	endian::AnyEndian,
	segment::ProgramHeader
};

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

	pub fn root(&self) -> PhysPageNum {
		self.page_table.root_ppn
	}

	/// Push a VMArea into the address space and optionally copy data
	fn push(&mut self, mut vma: VMArea, data: Option<&[u8]>){
		(&mut vma).map_all(&mut self.page_table);
		if let Some(data) = data {
			vma.copy_data(&self.page_table, data)
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

	pub fn insert_uflow_context(&mut self, flow: PhysAddr) -> VirtAddr {
		//TODO: check flow is aligned
		self.page_table.map(
			VirtPageNum::from_addr_floor(FLOW_CONTEXT_VADDR),
			flow.ppn_floor(),
			PTEFlags::R | PTEFlags::W,
			None
		);
		FLOW_CONTEXT_VADDR.into()
	}

	pub fn insert_utrap_handler(&mut self, traph: PhysAddr) -> VirtAddr {
		//TODO: check traph is aligned
		self.page_table.map(
			VirtPageNum::from_addr_floor(TRAP_HANDLER_VADDR),
			traph.ppn_floor(),
			PTEFlags::R | PTEFlags::W,
			None
		);
		TRAP_HANDLER_VADDR.into()
	}

	pub fn insert_uhart_context(&mut self, hc: PhysAddr) -> VirtAddr {
		//TODO: check hc is aligned
		self.page_table.map(
			VirtPageNum::from_addr_floor(HART_CONTEXT_VADDR),
			hc.ppn_floor(),
			PTEFlags::R | PTEFlags::W,
			None
		);
		TRAP_HANDLER_VADDR.into()
	}

	/// Create the kernel address space
	pub fn new_kernel() -> Self {
		let mut kernel_space = Self::new_bare();
		print_kernel_mem();
		// trampoline
		kernel_space.map_trampoline(VirtPageNum::from_addr_floor(TRAMPOLINE_VADDR));

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

		// mmio space
		PLATFORM.get().unwrap().board_info.get_all_ranges().iter().for_each(|range| {
			kernel_space.push(VMArea::new(
				range.start.into(),
				range.end.into(),
				MapType::Identical,
				MapPermission::R | MapPermission::W,
			), None);
		});
		kernel_space.print_addr_space();

		info!("kernel mapping address space build complete");
		kernel_space
	}

	pub fn map_trampoline(&mut self, vpn: VirtPageNum) {
		// we dont need vma because we dont use FrameAllocator
		let ppn = PhysPageNum::from_addr_floor(&raw const strampoline as usize);
		println!("vpn: 0x{:x}, ppn: 0x{:x}", vpn.0, ppn.0);
		self.page_table.map(
			vpn,
			ppn,
			PTEFlags::R | PTEFlags::X,
			None,
		);
	}

	/// Create address space from ELF, returning (self, user_sp, entry_point)
	/// rela file only
	pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
		let mut user_space = Self::new_bare();

		// trampoline
		user_space.map_trampoline(VirtPageNum::from_addr_floor(TRAMPOLINE_VADDR));

		// parse elf into ElfBytes
		let file = ElfBytes::<AnyEndian>::minimal_parse(elf_data).unwrap();
		let base_vaddr = if file.ehdr.e_type == ET_DYN {APP_VIRT_ADDR as u64} else {0};
		let load_phdr: Vec<ProgramHeader> = file.segments().unwrap()
			.iter()
			.filter(|phdr| { phdr.p_type == PT_LOAD })
			.collect();
		let end_vaddr =
			base_vaddr +
			load_phdr.last().unwrap().p_vaddr +
			load_phdr.last().unwrap().p_memsz;
		// base addr should be 0 so that size equal to end_vaddr
		let real_size = if file.ehdr.e_type == ET_DYN {end_vaddr} else {end_vaddr - APP_VIRT_ADDR as u64};
		let entry_point = if file.ehdr.e_type == ET_DYN {
			file.ehdr.e_entry as usize + APP_VIRT_ADDR
		} else {
			file.ehdr.e_entry as usize
		};

		// map every segments
		for phdr in load_phdr {
			let start_va: VirtAddr = ((phdr.p_vaddr + base_vaddr) as usize).into();
			let end_va: VirtAddr = ((phdr.p_vaddr + phdr.p_memsz + base_vaddr) as usize).into();
			let map_perm = MapPermission::from_elf_flags(phdr.p_flags);
			let data = elf_data.get(
				(phdr.p_offset as usize) .. ((phdr.p_offset + phdr.p_filesz) as usize)
			);

			let vma = VMArea::new(start_va, end_va, MapType::Framed, map_perm);
			user_space.push(vma, data);
		}
		// rela
		if file.ehdr.e_type == ET_DYN {
			let rela_dyn_header = file.section_header_by_name(".rela.dyn")
				.expect("section table should be parseable")
				.expect("should have .rela.dyn unless this elf file is not pie");
			let rela_dyn = file.section_data_as_relas(&rela_dyn_header)
				.expect("section data not found")
				.filter(|e| e.r_type == R_RISCV_RELATIVE);
			for entry in rela_dyn {
				unsafe {
					let offset =
						user_space.page_table.translate_vaddr((APP_VIRT_ADDR + entry.r_offset as usize).into())
							.unwrap()
							.0 as *mut i64; //TODO: should use virt addr?
					let append = APP_VIRT_ADDR as i64 + entry.r_addend;
					*offset = append;
				}
			}
		}

		// map user stack
		let end_va: VirtAddr = (end_vaddr as usize).into();
		let end_vpn = end_va.vpn_ceil();
		//guard page
		let user_stack_vpn: VirtPageNum = (end_vpn.0 + 1).into();
		let user_stack_va: VirtAddr = user_stack_vpn.into();
		let user_stack_va_end: VirtAddr = (user_stack_va.0 + USER_STACK_SIZE).into();
		let vma = VMArea::new(
			user_stack_va,
			user_stack_va_end,
			MapType::Framed,
			MapPermission::R | MapPermission::W | MapPermission::U
		);
		user_space.push(vma, None);
		user_space.print_addr_space();

		(user_space, user_stack_va_end.0, entry_point)
	}

	pub fn activate(&self) {
		self.page_table.activate_token();
	}

	pub fn token(&self) -> usize {
		self.page_table.token()
	}

	pub fn translated_byte_buffer(
		&self,
		ptr: *const u8,
		len: usize
	) -> Vec<&'static [u8]> {
		let start_va: VirtAddr = (ptr as usize).into();
		let start_vpn: VirtPageNum = start_va.vpn_floor();
		let end_va: VirtAddr = (ptr as usize + len).into();
		let end_vpn: VirtPageNum = end_va.vpn_ceil(); //.. not include end_vpn

		(start_vpn..end_vpn).map(|vpn| {
			let start_offset = if vpn == start_vpn {
				start_va.page_offset()
			} else {
				0
			};
			let step_next = VirtPageNum::forward_checked(vpn, 1).unwrap();
			let end_offset = if step_next == end_vpn {
				end_va.page_offset()
			} else {
				PAGE_SIZE
			};

			let ppn = self.page_table
				.translate_vpn(vpn)
				.unwrap(); //TODO: check the len and buf(may come from user space)
			unsafe {
				&ppn.get_byte_array()[start_offset..end_offset]
			}
		}).collect()
	}

	pub fn print_addr_space(&self) {
		info!("Address                      Permision  Map type");
		self.vma.iter().for_each(|vma| info!("{}", vma));
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
	pub fn copy_data(&mut self, pt_tree: &PageTableTree, data: &[u8]) {
		assert_eq!(self.map_type, MapType::Framed); // identical map can be directly access by vpn
		let mut cur_start: usize;
		let len = data.len();

		for (vpn, data_chunk) in self.vpn_range.clone().zip(data.chunks(PAGE_SIZE)) {
			let ppn = pt_tree.translate_vpn(vpn).unwrap(); //TODO: 需不需要对缺页的情况进行检查

			let dst = unsafe { ppn.get_byte_array() }; //SAFETY: this area will only be access by one cpu in one task
			let src = data_chunk;

			dst[..src.len()].copy_from_slice(src);
		}

	}

	/// Map a single page in the VMArea
	fn map_one(&mut self, pt_tree: &mut PageTableTree, vpn: VirtPageNum) {
		assert!(self.vpn_range.contains(&vpn));
		let (ppn, frame) = match self.map_type {
			MapType::Identical => {(PhysPageNum(vpn.0), None)},
			MapType::Framed => {
				let frame = FRAME_ALLOCATOR.get().unwrap().frame_alloc().unwrap();
				let ppn = frame.ppn;
				(ppn, Some(frame))
			}
		};
		pt_tree.map(vpn, ppn,
			PTEFlags::from_bits(self.map_perm.bits()).unwrap(), frame);
	}

	/// Unmap a single page in the VMArea
	fn unmap_one(&mut self, pt_tree: &mut PageTableTree, vpn: VirtPageNum) {
		pt_tree.unmap(vpn);
	}

}

impl Display for VMArea {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(f, "[0x{:<10x}, 0x{:<10x}]   {}{}{}{}     {}",
			VirtAddr::from(self.vpn_range.start).0,
    			VirtAddr::from(self.vpn_range.end).0,
			(if self.map_perm.contains(MapPermission::R) {"r"} else {"-"}),
			(if self.map_perm.contains(MapPermission::W) {"w"} else {"-"}),
			(if self.map_perm.contains(MapPermission::X) {"x"} else {"-"}),
			(if self.map_perm.contains(MapPermission::U) {"u"} else {"-"}),
			(if self.map_type == MapType::Framed {"Framed   "} else {"Identical"})
		)
	}
}

impl MapPermission {
	pub fn from_elf_flags(elf_pflags: u32) -> Self {
		Self::U
		| (elf_pflags & PF_R != 0).then_some(Self::R).unwrap_or_else(|| Self::empty())
		| (elf_pflags & PF_W != 0).then_some(Self::W).unwrap_or_else(|| Self::empty())
		| (elf_pflags & PF_X != 0).then_some(Self::X).unwrap_or_else(|| Self::empty())
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
		info!(".bss       : [{:<10p}, {:<10p}]", &sbss_nostack, &ebss);
		info!("kernel end = {:<10p}", &ekernel);
		let console = PLATFORM.get().unwrap().board_info.console.as_ref().unwrap();
		info!("console    : [{:<10x}, {:<10x}]", console.range.start, console.range.end);
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test_case]
	pub fn remap_test() {
		unsafe {
			let mut kernel_space = KERNEL_ADDRSPACE.get().unwrap();
			let mid_text: VirtAddr = ((&stext as *const _ as usize + &etext as *const _ as usize) / 2).into();
			let mid_rodata: VirtAddr = ((&srodata as *const _ as usize + &erodata as *const _ as usize) / 2).into();
			let mid_data: VirtAddr = ((&sdata as *const _ as usize + &edata as *const _ as usize) / 2).into();
			assert_eq!(
				kernel_space.page_table.translate(mid_text.vpn_floor()).unwrap().writable(),
				false
			);
			assert_eq!(
				kernel_space.page_table.translate(mid_rodata.vpn_floor()).unwrap().writable(),
				false,
			);
			assert_eq!(
				kernel_space.page_table.translate(mid_data.vpn_floor()).unwrap().executable(),
				false,
			);
			println!("remap_test passed!");
		}
	}
}