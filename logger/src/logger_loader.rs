use std::sync::Arc;
use crate::log_contracts::Logger;
use crate::log_contracts::LogLevel;
use crate::log_contracts::LogDestination;
use crate::log_config::LoggingConfig;
use crate::file_log_destination::FileLogDestination;
use crate::http_log_destination::HttpLogDestination;
use crate::LogWriter;

pub struct LoggerLoader;

impl LoggerLoader {
    pub fn load(config: &LoggingConfig) -> Arc<dyn Logger> {
        let log_dest: Arc<dyn LogDestination> = match config.r#type.as_str() {
            "http" => Arc::new(HttpLogDestination::new(
                config.http_endpoint.as_ref().expect("Missing http_endpoint")
            )),
            "file" => Arc::new(FileLogDestination::new(
                config.file_path.as_ref().expect("Missing file_path").into(),
                10 * 1024 * 1024,
            )),
            _ => panic!("Unknown logger type"),
        };

        let threshold = match config.threshold.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        };

        Arc::new(LogWriter::new(threshold, log_dest))
    }
}
