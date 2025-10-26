pub mod hart;
use core::arch::asm;
use crate::arch::riscv;
use crate::arch::loongarch64;

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv::entry;

#[cfg(target_arch = "loongarch64")]
pub use crate::arch::loongarch64::entry;

pub fn shutdown(fail: bool) -> ! {
  #[cfg(target_arch = "riscv64")]
  riscv::shutdown(fail);
  #[cfg(target_arch = "loongarch64")]
  loongarch64::shutdown(fail);
}

pub fn sleep(sec: i32) {
  #[cfg(target_arch = "riscv64")]
  riscv::sleep(sec);
  #[cfg(target_arch = "loongarch64")]
  loongarch64::sleep(sec);
}

pub unsafe fn fencei() {
  unsafe {
    #[cfg(target_arch = "riscv64")]
    asm!("fence.i");
    #[cfg(target_arch = "loongarch64")]
    asm!("ibar");
  }
}
