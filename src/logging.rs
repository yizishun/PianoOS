use log::{LevelFilter, Log, SetLoggerError, set_logger, set_max_level};
use log::Level;
use crate::sbi;

struct PianoLogger;
static PIANOLOGGER: PianoLogger = PianoLogger;
impl Log for PianoLogger {
    //only be use in log_enabled!, control the whole log
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }
    fn flush(&self) {}
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let ansi_color = match record.level() {
            Level::Info  => "\x1b[0;34m",
            Level::Error => "\x1b[0;31m",
            Level::Warn  => "\x1b[0;93m",
            Level::Debug => "\x1b[0;32m",
            Level::Trace => "\x1b[0;90m",
            _       => "\x1b[0m"
        };
        let hart_id = sbi::get_hartid();
        println!(
            "{} [{:<5}][{:<2}] {}\x1b[0m" , 
            ansi_color, 
            record.level(),
            hart_id,
            record.args()
        );

    }
}

pub fn init() -> Result<(), SetLoggerError> {
    set_logger(&PIANOLOGGER)?;
    set_max_level(match option_env!("LOG"){
        Some("INFO")  => LevelFilter::Info,
        Some("ERROR") => LevelFilter::Error,
        Some("WARN")  => LevelFilter::Warn,
        Some("TRACE") => LevelFilter::Trace,
        Some("DEBUG") => LevelFilter::Debug,
        Some("OFF")   => LevelFilter::Off,
        None | _ => LevelFilter::Info
    });
    Ok(())
}
