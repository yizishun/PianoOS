use sbi_rt::Physical;

use crate::console::ConsoleDevice;

pub struct RiscvSbi;

impl ConsoleDevice for RiscvSbi {
        /// read bytes from console input
        fn read(&self, buf: &mut [u8]) -> usize {
                sbi_rt::console_read(Physical::new(buf.len(),
                                                   buf.as_ptr() as usize,
                                                   buf[buf.len() - 1] as usize)).value
        }
        /// write bytes to console output
        fn write(&self, buf: &[u8]) -> usize {
                sbi_rt::console_write(Physical::new(buf.len(),
                                                    buf.as_ptr() as usize,
                                                    buf[buf.len() - 1] as usize)).value
        }
}
