#![macro_use]
use crate::sbi;
use core::fmt::{self, Write};

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().for_each(|c| {
            sbi::console_putchar(c as usize);
        });
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}

macro_rules! log_message {
    ($level: literal, $fmt: literal $(, $($arg: tt)+)?) => {
        let ansi_color = match $level {
            "INFO"  => "\x1b[0;34m",
            "ERROR" => "\x1b[0;31m",
            "WARN"  => "\x1b[0;93m",
            "DEBUG" => "\x1b[0;32m",
            "TRACE" => "\x1b[0;90m",
            _       => "\x1b[0m"
        };
        let hart_id = sbi::get_hartid();
        $crate::console::print(
            format_args!(
                concat!("{}", "[{:<5}][{:<2}] ", $fmt, "\x1b[0m", "\n") , ansi_color, $level, hart_id $(, $($arg)+)?
            )
        );
    };
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        log_message!("INFO", $fmt $(, $($arg)+)?);
    };
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        log_message!("ERROR", $fmt $(, $($arg)+)?);
    };
}

#[macro_export]
macro_rules! trace {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        log_message!("TRACE", $fmt $(, $($arg)+)?);
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        log_message!("WARN", $fmt $(, $($arg)+)?);
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        log_message!("DEBUG", $fmt $(, $($arg)+)?);
    };
}

