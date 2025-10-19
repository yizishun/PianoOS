pub mod hart;

use crate::FREQUNCY;
use log::info;
use riscv::register::{time, sie};

pub fn shutdown(fail: bool) -> ! {
    use sbi_rt::{NoReason, Shutdown, SystemFailure};
    if fail {
        sbi_rt::system_reset(Shutdown, SystemFailure);
    } else {
        info!("system will shutdown in 1 seconds");
        sleep(1);
        sbi_rt::system_reset(Shutdown, NoReason);
    }
    loop {}
}

pub fn sleep(sec: i32) {
    let time_start = time::read();
    let time_end = time_start + (FREQUNCY * 100_0000 * sec) as usize;
    unsafe {sie::set_stimer();}
    sbi_rt::set_timer(time_end as u64);
    riscv::asm::wfi();
}

