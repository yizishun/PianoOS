use crate::arch::{common::ArchISA, riscv::Riscv64};
use core::arch::asm;

impl<C> ArchISA for Riscv64<C> {
    unsafe fn fencei() {
        unsafe {
            asm!("fence.i");
        }
    }
}