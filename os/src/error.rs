use core::panic::PanicInfo;

use crate::{arch::riscv, devicetree::ParseDeviceTreeError};
use log::error;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
        if let Some(location) = _info.location() {
                error!("Panic at {}, line: {}, column: {}, due to {}.",
                       location.file(),
                       location.line(),
                       location.column(),
                       _info.message());
        } else {
                error!("Panic due to {}", _info.message());
        }
        riscv::shutdown(true);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelError {
        DeviceTree(ParseDeviceTreeError),
}

impl From<ParseDeviceTreeError> for KernelError {
        fn from(value: ParseDeviceTreeError) -> Self {
                Self::DeviceTree(value)
        }
}
