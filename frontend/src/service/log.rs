//! The log service

use failure::Error;
use log::{set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record};
use yew::services::ConsoleService;

/// The public static logger instance
static LOGGER: LogService = LogService;

/// Initialize the static logger
pub fn init_logger() -> Result<(), Error> {
    set_logger(&LOGGER)
        .map(|()| set_max_level(LevelFilter::Trace))
        .map_err(|_| format_err!("Logger init failed"))
}

/// The service used for logging purposes
struct LogService;

impl Log for LogService {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        // Verify that the logger is enabled
        if self.enabled(record.metadata()) {
            // Create a new ConsoleService
            let mut console_service = ConsoleService::new();

            // Create the log entry
            let mut log_entry = format!("{}: ", record.level());

            // Add file and line if available
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                log_entry += &format!("{}:{}: ", file, line);
            }

            // Add the body
            log_entry += &format!("{}", record.args());

            // Log the entry
            match record.level() {
                Level::Error => console_service.error(&log_entry),
                Level::Warn => console_service.warn(&log_entry),
                Level::Info => console_service.info(&log_entry),
                Level::Debug => console_service.debug(&log_entry),
                Level::Trace => console_service.debug(&log_entry),
            }
        }
    }

    fn flush(&self) {}
}
