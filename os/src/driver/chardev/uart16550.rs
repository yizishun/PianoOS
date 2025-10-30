use uart16550::{Register, Uart16550};

use crate::console::ConsoleDevice;
use crate::platform::BaseAddr;

pub struct Uart16550Wrapper<R: Register> {
        inner: *const Uart16550<R>,
}

impl<R: Register> Uart16550Wrapper<R> {
        pub fn new(base: BaseAddr) -> Self {
                Uart16550Wrapper { inner: base as *const Uart16550<R> }
        }
}

impl<R: Register> ConsoleDevice for Uart16550Wrapper<R> {
        fn read(&self, buf: &mut [u8]) -> usize {
                unsafe { (*self.inner).read(buf) }
        }
        fn write(&self, buf: &[u8]) -> usize {
                unsafe { (*self.inner).write(buf) }
        }
}

unsafe impl<R: Register> Send for Uart16550Wrapper<R> {}
