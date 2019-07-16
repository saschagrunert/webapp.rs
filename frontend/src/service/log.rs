//! The log service

use failure::{format_err, Fallible};
use log::{set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record};
use stdweb::js;

/// The public static logger instance
static LOGGER: LogService = LogService;

/// Initialize the static logger
pub fn init_logger() -> Fallible<()> {
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
            // Create the log entry
            let mut log_entry = format!("%c{}: %c", record.level());

            // Add file and line if available
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                log_entry += &format!("{}:{}: ", file, line);
            }

            // Add the body
            log_entry += &format!("{}", record.args());

            // Log the entry
            const BOLD: &str = "font-weight: bold";
            const NORMAL: &str = "font-weight: normal";
            match record.level() {
                Level::Error => {
                    js! { console.error(@{log_entry}, @{BOLD}, @{NORMAL}) }
                }
                Level::Warn => {
                    js! { console.warn(@{log_entry}, @{BOLD}, @{NORMAL}) }
                }
                Level::Info => {
                    js! { console.log(@{log_entry}, @{BOLD}, @{NORMAL}) }
                }
                Level::Debug => {
                    js! { console.debug(@{log_entry}, @{BOLD}, @{NORMAL}) }
                }
                Level::Trace => {
                    js! { console.debug(@{log_entry}, @{BOLD}, @{NORMAL}) }
                }
            }
        }
    }

    fn flush(&self) {}
}
