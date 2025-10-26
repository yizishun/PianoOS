pub mod riscv;
pub mod loongarch64;
pub mod hart;

use core::arch::asm;

#[cfg(target_arch = "riscv64")]
pub use riscv::entry;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::entry;

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

