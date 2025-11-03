use crate::harts::current_hartid_in_boot_stage;
use crate::println;
use log::Level;
use log::{LevelFilter, Log, SetLoggerError, set_logger, set_max_level};

struct PianoLogger;
static PIANOLOGGER: PianoLogger = PianoLogger;
impl Log for PianoLogger {
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
		let hart_id = current_hartid_in_boot_stage();
		println!("{bold}[kernel]{reset}{color_log} {:<5}[{:>2}]{reset} - {}",
			 record.level(),
			 hart_id,
			 record.args(),
			 reset = ansi_reset,
			 color_log = ansi_color,
			 bold = bold);
//                println!("{bold}[kernel]{reset}{color_log} {:<5}{reset} - {}",
//                         record.level(),
//                         record.args(),
//                         reset = ansi_reset,
//                         color_log = ansi_color,
//                         bold = bold);
	}
}

pub fn init() -> Result<(), SetLoggerError> {
	set_logger(&PIANOLOGGER)?;
	set_max_level(match option_env!("LOG") {
			      Some("INFO") => LevelFilter::Info,
			      Some("ERROR") => LevelFilter::Error,
			      Some("WARN") => LevelFilter::Warn,
			      Some("TRACE") => LevelFilter::Trace,
			      Some("DEBUG") => LevelFilter::Debug,
			      Some("OFF") => LevelFilter::Off,
			      None | _ => LevelFilter::Info,
		      });
	Ok(())
}
