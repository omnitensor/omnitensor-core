use log::{Record, Level, Metadata};
use std::env;

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_logger(default_level: &str) {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| default_level.to_string());
    let level = match log_level.to_lowercase().as_str() {
        "debug" => Level::Debug,
        "info" => Level::Info,
        "warn" => Level::Warn,
        "error" => Level::Error,
        _ => Level::Info,
    };
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(level.to_level_filter());
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;

    #[test]
    fn test_logger() {
        init_logger("info");
        info!("Logger initialized successfully.");
    }
}
