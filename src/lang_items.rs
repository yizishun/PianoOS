
use core::panic::PanicInfo;

use crate::{println, sbi};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        println!("Panic at {}, line: {}, column: {}, due to {}.", 
            location.file(),
            location.line(),
            location.column(),
            _info.message()
        );
    } else {
        println!("Panic due to {}", _info.message());
    }
    sbi::shutdown(true);
}