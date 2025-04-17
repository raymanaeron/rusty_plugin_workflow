use std::sync::{ Arc, OnceLock };
use crate::log_contracts::{ LogLevel, LogDestination };
use crate::log_config::LoggingConfig;
use crate::file_log_destination::FileLogDestination;
use crate::http_log_destination::HttpLogDestination;
use crate::LogWriter;
use crate::Logger;
use crate::TelemetryConfig;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use toml;

pub struct LoggerLoader;

pub static LOGGER: OnceLock<Arc<dyn Logger>> = OnceLock::new();

impl LoggerLoader {
    pub fn load(config: &LoggingConfig) -> Arc<dyn Logger> {
        let log_dest: Arc<dyn LogDestination> = match config.r#type.as_str() {
            "http" =>
                Arc::new(
                    HttpLogDestination::new(
                        config.http_endpoint.as_ref().expect("Missing http_endpoint")
                    )
                ),
            "file" =>
                Arc::new(
                    FileLogDestination::new(
                        config.file_path.as_ref().expect("Missing file_path").into(),
                        10 * 1024 * 1024
                    )
                ),
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

    pub async fn init(config_path: &str) {
        // Read configuration from the provided path
        let mut config_file = File::open(config_path).await.expect(
            "Unable to open configuration file"
        );
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).await.expect("Failed to read configuration file");

        // Parse the TOML configuration into TelemetryConfig
        let telemetry_config: TelemetryConfig = toml
            ::from_str(&contents)
            .expect("Error parsing configuration file");

        // Load the logger using the logging configuration
        let logger = LoggerLoader::load(&telemetry_config.logging);

        // Store the logger in the global LOGGER variable
        LOGGER.set(logger).expect("Logger already initialized");
    }

    pub fn get_logger() -> &'static Arc<dyn Logger> {
        LOGGER.get().expect("Logger is not initialized")
    }
}
