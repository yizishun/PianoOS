use super::write;
use core::fmt::{self, Error, Write};

struct Stdout;

const STDOUT: usize = 1;

impl Write for Stdout {
        fn write_str(&mut self, s: &str) -> fmt::Result {
                let size = write(STDOUT, s.as_bytes()) as usize;
                if size < s.as_bytes().len() {
                    Err(Error)
                } else {
                    Ok(())
                }
        }
}

pub fn print(args: fmt::Arguments) {
        Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
