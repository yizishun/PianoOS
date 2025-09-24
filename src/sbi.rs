use riscv::register::time;
use crate::FREQUNCY;

pub fn console_putchar(c: usize) { //TODO: error handling
    c
    .to_le_bytes()
    .iter()
    .for_each( |c_bytes| {
            sbi_rt::console_write_byte(*c_bytes);
        }
    );
}

pub fn shutdown(fail: bool) -> !{
    use sbi_rt::{Shutdown, NoReason, SystemFailure};
    if fail {
        sbi_rt::system_reset(Shutdown, SystemFailure);
    }else {
        sbi_rt::system_reset(Shutdown, NoReason);
    }
    unreachable!()
}

pub fn sleep(sec: i32) {
    let time_start = time::read();
    let time_end = time_start + ( FREQUNCY * 100_0000 * sec ) as usize;
    sbi_rt::set_timer(time_end as u64);
    riscv::asm::wfi();
}
