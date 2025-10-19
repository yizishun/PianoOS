#![macro_use]
#![allow(static_mut_refs)]
use core::fmt::{self, Write};
use alloc::boxed::Box;
use spin::Mutex;

use crate::platform::PLATFORM;

#[derive(Clone, Copy)]
pub enum ConsoleType {
    Uart16550U8,
    Uart16550U32,
    RiscvSbi
}

impl ConsoleType {
    pub fn compatible(device: &str) -> Option<ConsoleType>{
        use ConsoleType::*;
        if        ["ns16550a"].contains(&device) { 
            Some(Uart16550U8)
        } else if ["snps,dw-apb-uart"].contains(&device) {
            Some(Uart16550U32)
        } else {
            None
        }
    }
}

/// console device driver should impl this trait 
pub (in crate)trait ConsoleDevice {
    /// read bytes from console input
    fn read(&self, buf: &mut [u8]) -> usize;
    /// write bytes to console output
    fn write(&self, buf: &[u8]) -> usize;
}

pub struct KernelConsole {
    inner: Mutex<Box<dyn ConsoleDevice>> //spinlock
}

impl KernelConsole {
    pub fn new(inner: Mutex<Box<dyn ConsoleDevice>>) -> Self {
        Self { inner }
    }
}

impl fmt::Write for KernelConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let console = self.inner.lock();
        let mut bytes = s.as_bytes();
        while !bytes.is_empty() {
            let count = console.write(bytes);
            bytes = &bytes[count..];
        }
        Ok(()) //TODO: error handle
    }
}

pub fn print(args: fmt::Arguments) {
    unsafe {
        PLATFORM.board_device.console.as_mut().unwrap()
            .write_fmt(args).unwrap();
    }
}

