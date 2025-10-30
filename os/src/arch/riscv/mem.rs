use crate::arch::{common::ArchMem, riscv::Riscv64};
use core::arch::asm;

impl<C> ArchMem for Riscv64<C> {
        unsafe fn fencei(&self) {
                unsafe {
                        asm!("fence.i");
                }
        }
}
