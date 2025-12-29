use crate::harts::{hart_id_in_trap_stage, task_context_in_trap_stage};
use crate::println;
use alloc::boxed::Box;
use log::Level;
use log::{LevelFilter, Log, SetLoggerError, set_logger, set_max_level};
use spin::RwLock;
use spin::Once;

pub struct PianoLogger {
	inner: RwLock<Box<dyn Log + Send + Sync>>
}

struct BootLogger;
struct TrapLogger;

impl PianoLogger {
	pub fn set_boot_logger() -> Self{
		Self { inner: RwLock::new(Box::new(BootLogger)) }
	}
	pub fn set_trap_logger(&self){
		*self.inner.write() = Box::new(TrapLogger)
	}
}
pub static PIANOLOGGER: Once<PianoLogger> = Once::new();

impl Log for PianoLogger {
	//only be use in log_enabled!, control the whole log
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		self.inner.read().enabled(metadata)
	}
	fn flush(&self) {
		self.inner.read().flush();
	}
	fn log(&self, record: &log::Record) {
		self.inner.read().log(record);
	}
}


impl Log for BootLogger {
	//only be use in log_enabled!, control the whole log
	fn enabled(&self, _metadata: &log::Metadata) -> bool {
		true
	}
	fn flush(&self) {}
	fn log(&self, record: &log::Record) {
		if !self.enabled(record.metadata()) {
			return;
		}
		let ansi_color = match record.level() {
			Level::Info => "\x1b[1;34m",
			Level::Error => "\x1b[1;31m",
			Level::Warn => "\x1b[1;33m",
			Level::Debug => "\x1b[1;32m",
			Level::Trace => "\x1b[1;90m",
		};
		let ansi_reset = "\x1b[0m";
		let bold = "\x1b[1;37m";
		println!("{bold}[kernel]{reset}{color_log} {:<5}{reset} - {}",
			 record.level(),
			 record.args(),
			 reset = ansi_reset,
			 color_log = ansi_color,
			 bold = bold);
	}
}

impl Log for TrapLogger {
	//only be use in log_enabled!, control the whole log
	fn enabled(&self, _metadata: &log::Metadata) -> bool {
		true
	}
	fn flush(&self) {}
	fn log(&self, record: &log::Record) {
		if !self.enabled(record.metadata()) {
			return;
		}
		let ansi_color = match record.level() {
			Level::Info => "\x1b[1;34m",
			Level::Error => "\x1b[1;31m",
			Level::Warn => "\x1b[1;33m",
			Level::Debug => "\x1b[1;32m",
			Level::Trace => "\x1b[1;90m",
		};
		let ansi_reset = "\x1b[0m";
		let bold = "\x1b[1;37m";
		let hart_id = hart_id_in_trap_stage();
		let app_id = task_context_in_trap_stage().app_info().app_id;
		println!("{bold}[kernel]{reset}{color_log} {:<5}[{:>2}][{:>2}]{reset} - {}",
			 record.level(),
			 hart_id,
			 app_id,
			 record.args(),
			 reset = ansi_reset,
			 color_log = ansi_color,
			 bold = bold);
	}
}

pub fn init() -> Result<(), SetLoggerError> {
	set_logger(PIANOLOGGER.get().unwrap())?;
	set_max_level(match option_env!("LOG") {
			      Some("INFO") => LevelFilter::Info,
			      Some("ERROR") => LevelFilter::Error,
			      Some("WARN") => LevelFilter::Warn,
			      Some("TRACE") => LevelFilter::Trace,
			      Some("DEBUG") => LevelFilter::Debug,
			      Some("OFF") => LevelFilter::Off,
			      None | _ => LevelFilter::Trace,
		      });
	Ok(())
}
