use alloc::vec::Vec;
use core::{ops::Range, ptr::copy_nonoverlapping};
use elf::{
	ElfBytes,
	abi::{PT_LOAD, R_RISCV_RELATIVE},
	endian::AnyEndian,
	segment::ProgramHeader
};
use crate::config::MAX_APP_NUM;
use crate::_num_app;
use crate::info;

use crate::{arch::common::ArchMem, global::ARCH};

pub struct LoaderElfInfo {
	pub num_app: usize,
	elf_info: [ElfInfo; MAX_APP_NUM]
}

impl LoaderElfInfo {
	pub fn new() -> Self {
		let num_app_ptr: *const usize = core::ptr::addr_of!(_num_app);
		let num_app_usize: usize = unsafe { *num_app_ptr };
		let count: usize = num_app_usize + 1;
		let app_start_addr_raw: &[usize] =
			unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), count) };
		let mut app_start_addr: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
		app_start_addr[..count].copy_from_slice(app_start_addr_raw);
		let mut elf_info: [ElfInfo; MAX_APP_NUM] = [ElfInfo::ZERO; MAX_APP_NUM];
		for (elf, i) in elf_info.iter_mut().zip(0..num_app_usize) {
			elf.start_addr = app_start_addr[i];
			elf.end_addr = app_start_addr[i + 1];
		}
		LoaderElfInfo {
			num_app: num_app_usize,
			elf_info
		}
	}

	pub fn elf_info(&self, idx: usize) -> &ElfInfo {
		self.elf_info.get(idx).unwrap()
	}

	pub fn print_app_info(&self) {
		info!("Kernel app number: {}", self.num_app);
		for i in 0..self.num_app {
			info!("app {i}: [{:<10p}, {:<10p}]",
			      self.elf_info[i].start_addr as *const usize,
			      self.elf_info[i].end_addr as *const usize);
		}
	}

	pub fn app_size(&self, app_id: usize) -> usize {
		assert!(app_id < self.num_app, "Invalid app id {}", app_id);
		let size: isize = (self.elf_info[app_id].end_addr - self.elf_info[app_id].start_addr) as isize;
		assert!(size >= 0, "app size is nagative");
		size as usize
	}

	pub fn load(&self, app_id: usize) -> Range<*const u8> {
		use crate::config::APP_BASE_ADDR;
		assert!(app_id < self.num_app, "app id {} is greater than number of app {}", app_id, self.num_app);

		let elf = self.elf_info.get(app_id).unwrap();
		let off = elf.start_addr - self.elf_info.get(0).unwrap().start_addr;
		let dst = (APP_BASE_ADDR + off) as *const u8;
		elf.load_elf(dst)
	}

}

pub struct ElfInfo {
	pub start_addr: usize,
	pub end_addr: usize
}

impl ElfInfo {
	pub const ZERO: ElfInfo = ElfInfo {
		start_addr: 0,
		end_addr: 0
	};

	pub fn load_elf(&self, dst: *const u8) -> Range<*const u8> {
		let count = self.end_addr - self.start_addr;
		let dst_start = dst;
		let slice = unsafe {
			core::slice::from_raw_parts(self.start_addr as *const u8, count)
		};


		let file = ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();
		let load_phdr: Vec<ProgramHeader> = file.segments().unwrap()
			.iter()
			.filter(|phdr| { phdr.p_type == PT_LOAD })
			.collect();
		// base addr should be 0 so that size equal to p_vaddr
		let real_size = 
			load_phdr.last().unwrap().p_vaddr + 
			load_phdr.last().unwrap().p_memsz;
		// load!
		for phdr in load_phdr {
			let dst = unsafe {
				dst_start.byte_add(phdr.p_vaddr as usize) as *mut u8
			};
			let src = unsafe {
				(self.start_addr as *const u8).byte_add(phdr.p_offset as usize)
			};
			let count = phdr.p_filesz;
			unsafe {
				copy_nonoverlapping(src, dst, count as usize);
				let lap = core::slice::from_raw_parts_mut(
					dst.byte_add(phdr.p_filesz as usize), 
					(phdr.p_memsz - phdr.p_filesz) as usize);
				lap.fill(0);
				ARCH.fencei();
			}
		}
		// rela!
		let rela_dyn_header = file.section_header_by_name(".rela.dyn")
			.expect("section table should be parseable")
			.expect("should have .rela.dyn unless this elf file is not pie");
		let rela_dyn = file.section_data_as_relas(&rela_dyn_header)
			.expect("section data not found")
			.filter(|e| e.r_type == R_RISCV_RELATIVE);
		for entry in rela_dyn {
			unsafe {
				let offset = dst_start.byte_add(entry.r_offset as usize) as *mut usize;
				let append = dst_start.byte_add(entry.r_addend as usize) as usize;
				*offset = append;
				ARCH.fencei();
			}

		}

		unsafe {
			dst_start..dst_start.byte_add(real_size as usize)
		}
	}
}
