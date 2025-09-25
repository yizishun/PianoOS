use core::panic::PanicInfo;

use crate::sbi;
use log::error;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        error!(
            "Panic at {}, line: {}, column: {}, due to {}.",
            location.file(),
            location.line(),
            location.column(),
            _info.message()
        );
    } else {
        error!("Panic due to {}", _info.message());
    }
    sbi::shutdown(true);
}
