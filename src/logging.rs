extern crate log;

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;
static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    if cfg!(debug_assertions) {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
    } else {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if cfg!(debug_assertions) {
            metadata.level() <= Level::Trace
        } else {
            metadata.level() <= Level::Info
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            use cortex_m_semihosting::hprintln;
            hprintln!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
