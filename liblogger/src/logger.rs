/*
 * Logger implementation module
 * 
 * This file implements the core Logger functionality which includes:
 * - Creation and initialization of the global logger instance
 * - Configuration of the logger from TOML files or programmatically
 * - Methods for logging messages at different severity levels
 * - Thread-safe logging with proper synchronization
 * 
 * The Logger uses a singleton pattern with lazy initialization via OnceCell
 * to ensure there's only one logger instance throughout the application.
 */

use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};
use std::path::Path;
use chrono::Utc;
use std::io::{self, Write};

use crate::config::{LogConfig, LogLevel};
use crate::outputs::{LogOutput, create_log_output};

static LOGGER_INSTANCE: OnceCell<Arc<Mutex<LoggerInner>>> = OnceCell::new();

struct LoggerInner {
    initialized: bool,
    config: Option<LogConfig>,
    output: Option<Box<dyn LogOutput>>,
}

impl LoggerInner {
    fn new() -> Self {
        LoggerInner {
            initialized: false,
            config: None,
            output: None,
        }
    }

    fn init_with_config(&mut self, config: LogConfig) -> Result<(), String> {
        self.config = Some(config.clone());
        self.output = Some(create_log_output(&config)?);
        self.initialized = true;
        Ok(())
    }

    fn log(&mut self, level: LogLevel, message: &str, context: Option<&str>, file: &str, line: u32, module: &str) {
        // If not initialized, log to stderr as fallback
        if !self.initialized {
            let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let log_line = if let Some(ctx) = context {
                format!("{} [{}] [{}:{}] [{}] {} | Context: {}\n", 
                    timestamp, level.as_str(), file, line, module, message, ctx)
            } else {
                format!("{} [{}] [{}:{}] [{}] {}\n", 
                    timestamp, level.as_str(), file, line, module, message)
            };
            let _ = io::stderr().write_all(log_line.as_bytes());
            return;
        }

        // Check if we should log this level
        if let Some(config) = &self.config {
            if level.should_log(&config.threshold) {
                let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                if let Some(output) = &mut self.output {
                    if let Err(e) = output.write_log(&timestamp, &level, message, file, line, module, context) {
                        // If logging fails, write to stderr as fallback
                        let _ = writeln!(io::stderr(), "Logging failed: {}. Original message: {}", e, message);
                    }
                }
            }
        }
    }
}

pub struct Logger;

impl Logger {
    /// Initialize the logger with default configuration file "app_config.toml"
    pub fn init() {
        let _ = Self::init_with_config_file("app_config.toml");
    }

    /// Initialize the logger with a specific configuration file
    pub fn init_with_config_file(config_path: &str) -> Result<(), String> {
        println!("Initializing logger with config file: {}", config_path);
        let config = match LogConfig::from_file(config_path) {
            Ok(cfg) => {
                println!("Config loaded successfully. Log type: {:?}", cfg.log_type);
                cfg
            },
            Err(e) => {
                println!("Error loading config: {}", e);
                return Err(e);
            }
        };
        Self::init_with_config(config)
    }

    /// Initialize the logger with a LogConfig struct
    pub fn init_with_config(config: LogConfig) -> Result<(), String> {
        println!("Setting up logger with log type: {:?}", config.log_type);
        
        let logger = LOGGER_INSTANCE.get_or_init(|| Arc::new(Mutex::new(LoggerInner::new())));
        let mut logger_guard = match logger.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                println!("Logger mutex was poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        
        match logger_guard.init_with_config(config) {
            Ok(_) => {
                println!("Logger initialized successfully");
                Ok(())
            },
            Err(e) => {
                println!("Failed to initialize logger: {}", e);
                Err(e)
            }
        }
    }

    /// Log a debug message
    pub fn debug(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Debug, message, context, file, line, module)
    }

    /// Log an info message
    pub fn info(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Info, message, context, file, line, module)
    }

    /// Log a warning message
    pub fn warn(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Warn, message, context, file, line, module)
    }

    /// Log an error message
    pub fn error(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Error, message, context, file, line, module)
    }

    fn log_with_metadata(level: LogLevel, message: &str, context: Option<String>, file: &str, line: u32, module: &str) {
        // Extract just the filename from the path
        let file_name = Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file);

        let logger = LOGGER_INSTANCE.get_or_init(|| Arc::new(Mutex::new(LoggerInner::new())));
        
        // Use a block to limit the scope of the mutex lock
        {
            if let Ok(mut logger) = logger.lock() {
                logger.log(level, message, context.as_deref(), file_name, line, module);
            } else {
                // If the mutex is poisoned, log to stderr
                let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                let log_line = format!("{} [{}] [{}:{}] [{}] {} | MUTEX POISONED\n",
                    timestamp, level.as_str(), file_name, line, module, message);
                let _ = io::stderr().write_all(log_line.as_bytes());
            }
        }
    }
}
