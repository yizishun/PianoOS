use riscv::asm::fence_i;

use crate::arch::{common::ArchMem, riscv::Riscv64};
use core::arch::asm;

impl<C> ArchMem for Riscv64<C> {
        unsafe fn fencei(&self) {
                fence_i();
        }
}
