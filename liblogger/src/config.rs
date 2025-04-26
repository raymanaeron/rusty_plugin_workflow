/*
 * Configuration management for the logger
 * 
 * This module handles:
 * - Parsing configuration from TOML files
 * - Defining the LogType enum (Console, File, Http)
 * - Defining the LogLevel enum (Debug, Info, Warn, Error)
 * - Implementing methods for level comparison and string conversion
 * - Providing default configuration values
 * 
 * The configuration determines where logs are written, their format,
 * and which severity levels are included in the output.
 */

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub enum LogType {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "http")]
    Http,
}

#[derive(Debug, Clone, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
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

    pub fn from_str(s: &str) -> LogLevel {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info, // Default to info for unknown levels
        }
    }

    pub fn should_log(&self, threshold: &LogLevel) -> bool {
        match threshold {
            // If threshold is Debug, log everything
            LogLevel::Debug => true,
            
            // If threshold is Info, log Info, Warn, Error but not Debug
            LogLevel::Info => match self {
                LogLevel::Debug => false,
                _ => true,
            },
            
            // If threshold is Warn, log only Warn and Error
            LogLevel::Warn => match self {
                LogLevel::Debug | LogLevel::Info => false,
                _ => true,
            },
            
            // If threshold is Error, log only Error
            LogLevel::Error => match self {
                LogLevel::Error => true,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(rename = "type")]
    pub log_type: LogType,
    pub threshold: LogLevel,
    #[serde(default = "default_file_path")]
    pub file_path: String,
    #[serde(default = "default_log_folder")]
    pub log_folder: String,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    #[serde(default = "default_http_endpoint")]
    pub http_endpoint: String,
    #[serde(default = "default_http_timeout")]
    pub http_timeout_seconds: u64,
}

fn default_file_path() -> String {
    "app.log".into()
}

fn default_log_folder() -> String {
    "logs".into()
}

fn default_max_file_size() -> u64 {
    10 // 10 MB by default
}

fn default_http_endpoint() -> String {
    "http://localhost:8080/logs".into()
}

fn default_http_timeout() -> u64 {
    5 // 5 seconds by default
}

impl LogConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: toml::Table = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        let logging_section = config.get("logging")
            .ok_or_else(|| "Missing [logging] section in config".to_string())?;
        
        let log_config: LogConfig = toml::from_str(&toml::to_string(logging_section).unwrap())
            .map_err(|e| format!("Failed to parse logging config: {}", e))?;
        
        Ok(log_config)
    }

    pub fn ensure_log_folder_exists(&self) -> Result<(), String> {
        // Always create the log folder, regardless of log type
        let path = Path::new(&self.log_folder);
        if !path.exists() {
            println!("[Config] Creating log directory: {:?}", path);
            fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create log directory: {}", e))?;
        }
        Ok(())
    }
}
