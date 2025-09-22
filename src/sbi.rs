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