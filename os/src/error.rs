use core::panic::PanicInfo;

use crate::arch::common::ArchPower;
use crate::devicetree::ParseDeviceTreeError;
use crate::syscall::syscallid::SyscallError;
use crate::global::ARCH;
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
	ARCH.shutdown(true);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelError {
        DeviceTree(ParseDeviceTreeError),
        Syscall(SyscallError)
}

impl From<ParseDeviceTreeError> for KernelError {
        fn from(value: ParseDeviceTreeError) -> Self {
                Self::DeviceTree(value)
        }
}
