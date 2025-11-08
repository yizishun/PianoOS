use alloc::vec::Vec;
use core::{ops::Range, ptr::copy_nonoverlapping};
use elf::{
	ElfBytes,
	abi::{PT_LOAD, R_RISCV_RELATIVE},
	endian::AnyEndian,
	segment::ProgramHeader
};

use crate::{arch::common::ArchMem, global::{APP_MANAGER, ARCH}};

pub struct ElfInfo {
	pub start_addr: usize,
	pub end_addr: usize
}

impl ElfInfo {
	pub const ZERO: ElfInfo = ElfInfo {
		start_addr: 0,
		end_addr: 0
	};

	pub fn load_elf(&self) -> Range<*const u8> {
		use crate::config::APP_BASE_ADDR;
		let app_addr_off = self.start_addr - APP_MANAGER.get().unwrap().elf_info(0).start_addr;
		let count = self.start_addr - self.end_addr;
		let dst_start = (APP_BASE_ADDR + app_addr_off) as *const u8;
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
