use serde::Deserialize;
use std::sync::Arc;
use crate::log_contracts::Logger;
use crate::log_contracts::MetricsEmitter;

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub r#type: String, // "file" or "http"
    pub file_path: Option<String>,
    pub http_endpoint: Option<String>,
    pub threshold: String, // e.g. "debug", "info"
}

#[derive(Debug, Deserialize)]
pub struct MetricsConfig {
    pub r#type: String, // "file" or "http"
    pub file_path: Option<String>,
    pub http_endpoint: Option<String>,
}

/// This is the one that will be used in the engine
/// to load the logger and metrics emitter.
#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}

pub trait TelemetryConfigLoader {
    fn load(config_path: &str) -> (Arc<dyn Logger>, Arc<dyn MetricsEmitter>);
}
