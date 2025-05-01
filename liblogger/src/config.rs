/*
 * Configuration management for the Rusty Logger v2
 * 
 * This module handles:
 * - Parsing configuration from TOML files (app_config.toml)
 * - Defining the LogType enum for output destinations (Console, File, Http)
 * - Defining the LogLevel enum for severity levels (Debug, Info, Warn, Error)
 * - Implementing methods for level comparison and string conversion
 * - Providing default configuration values for all settings
 * 
 * The configuration determines:
 * - Where logs are written (console, file with rotation, or HTTP endpoint)
 * - Which severity levels are included in the output based on threshold
 * - File paths, rotation sizes, and HTTP timeouts
 * - Behavior of both synchronous and asynchronous logging operations
 */

 use serde::{Deserialize, Serialize};
 use std::fs;
 use once_cell::sync::OnceCell;
 
 /// Log severity levels
 #[derive(Debug, Clone, PartialEq, Serialize)]
 pub enum LogLevel {
     Debug,
     Info,
     Warn,
     Error,
 }
 
 // Separate implementation of Deserialize to handle case-insensitive values
 impl<'de> Deserialize<'de> for LogLevel {
     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
     where
         D: serde::Deserializer<'de>,
     {
         let s = String::deserialize(deserializer)?;
         match s.to_lowercase().as_str() {
             "debug" => Ok(LogLevel::Debug),
             "info" => Ok(LogLevel::Info),
             "warn" | "warning" => Ok(LogLevel::Warn),
             "error" => Ok(LogLevel::Error),
             _ => Err(serde::de::Error::unknown_variant(
                 &s,
                 &["debug", "info", "warn", "warning", "error"],
             )),
         }
     }
 }
 
 impl LogLevel {
     pub fn as_str(&self) -> &'static str {
         match self {
             LogLevel::Debug => "DEBUG",
             LogLevel::Info => "INFO",
             LogLevel::Warn => "WARN",
             LogLevel::Error => "ERROR",
         }
     }
 }
 
 /// Supported output types for logging
 #[derive(Debug, Clone, PartialEq, Serialize)]
 pub enum LogType {
     Console,
     File,
     Http,
 }
 
 // Separate implementation of Deserialize to handle case-insensitive values
 impl<'de> Deserialize<'de> for LogType {
     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
     where
         D: serde::Deserializer<'de>,
     {
         let s = String::deserialize(deserializer)?;
         match s.to_lowercase().as_str() {
             "console" => Ok(LogType::Console),
             "file" => Ok(LogType::File),
             "http" => Ok(LogType::Http),
             _ => Err(serde::de::Error::unknown_variant(
                 &s,
                 &["console", "file", "http"],
             )),
         }
     }
 }
 
 static CONFIG_INSTANCE: OnceCell<LogConfig> = OnceCell::new();
 
 /// Configuration for the logger
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct LogConfig {
     /// Type of output (console, file, http)
     #[serde(rename = "type")]
     pub log_type: LogType,
     
     /// Minimum log level to record
     pub threshold: LogLevel,
     
     /// File path for file-based logging
     #[serde(default)]
     pub file_path: Option<String>,
     
     /// Folder for log files
     #[serde(default)]
     pub log_folder: Option<String>,
     
     /// Maximum file size before rotation (in MB)
     #[serde(default)]
     pub max_file_size_mb: Option<u64>,
     
     /// Endpoint URL for HTTP logging
     #[serde(default)]
     pub http_endpoint: Option<String>,
     
     /// Timeout in seconds for HTTP requests
     #[serde(default)]
     pub http_timeout_seconds: Option<u64>,
     
     /// Whether to use async logging (default: true)
     #[serde(default = "default_async_logging")]
     pub async_logging: bool,
     
     /// Whether to force flush after every write (default: false)
     #[serde(default = "default_force_flush")]
     pub force_flush: bool,
 }
 
 fn default_async_logging() -> bool {
     true
 }
 
 fn default_force_flush() -> bool {
     false  // Default to false for better performance
 }
 
 impl Default for LogConfig {
     fn default() -> Self {
         LogConfig {
             log_type: LogType::Console,
             threshold: LogLevel::Info,
             file_path: None,
             log_folder: None,
             max_file_size_mb: None,
             http_endpoint: None,
             http_timeout_seconds: None,
             async_logging: true,
             force_flush: false,
         }
     }
 }
 
 /// Configuration wrapper to handle the [logging] section in TOML
 #[derive(Debug, Clone, Serialize, Deserialize)]
 struct ConfigWrapper {
     logging: LogConfig
 }
 
 impl LogConfig {
     /// Create configuration from a TOML file
     pub fn from_file(file_path: &str) -> Result<Self, String> {
         let config_str = match fs::read_to_string(file_path) {
             Ok(content) => content,
             Err(e) => {
                 println!("Warning: Could not read config file '{}': {}. Using defaults.", file_path, e);
                 return Ok(LogConfig::default());
             }
         };
 
         // Try to parse with the [logging] section wrapper first
         let config = match toml::from_str::<ConfigWrapper>(&config_str) {
             Ok(wrapper) => wrapper.logging,
             Err(e) => {
                 // If that fails, try the old format (direct LogConfig)
                 match toml::from_str::<LogConfig>(&config_str) {
                     Ok(config) => config,
                     Err(_) => {
                         // Return the original error if both parsing attempts fail
                         return Err(format!("Failed to parse config file: {}", e));
                     }
                 }
             }
         };
 
         // Set the global instance
         let _ = CONFIG_INSTANCE.get_or_init(|| config.clone());
         
         Ok(config)
     }
     
     /// Get the global instance of LogConfig
     pub fn get_instance() -> Result<LogConfig, String> {
         match CONFIG_INSTANCE.get() {
             Some(config) => Ok(config.clone()),
             None => Err("LogConfig not initialized. Call LogConfig::from_file first.".into())
         }
     }
 }
 