/*
 * Main library entry point for Rusty Logger v2
 * 
 * This file defines the public interface for the logging library, including:
 * - Re-exporting the Logger struct for initialization and configuration
 * - Re-exporting LogConfig, LogLevel, and LogType for custom configuration
 * - Defining logging macros (log_debug, log_info, log_warn, log_error)
 * - Providing a shutdown function for graceful termination of async logging
 * 
 * The library supports both synchronous and asynchronous logging operations
 * with multiple output targets (console, file, HTTP).
 */

 mod config;
 mod outputs;
 mod logger;
 
 /// Main logger class that handles initialization and log operations
 /// 
 /// Use this to initialize the logger with a configuration file or defaults.
 /// Example: `Logger::init_with_config_file("app_config.toml")`
 pub use logger::Logger;
 
 /// Configuration structures for customizing logger behavior
 /// 
 /// - LogConfig: Main configuration struct with all settings
 /// - LogLevel: Enum for severity levels (Debug, Info, Warn, Error)
 pub use config::{LogConfig, LogLevel};
 
 /// Enum defining available output destinations
 /// 
 /// - Console: Logs to standard output
 /// - File: Logs to a file with rotation
 /// - Http: Sends logs to a remote endpoint
 pub use config::LogType;
 
 /// Log a debug-level message
 /// 
 /// # Example
 /// ```
 /// log_debug!("Connection pool initialized with 10 connections");
 /// log_debug!("User authenticated", Some(format!("user_id={}", user_id)));
 /// ```
 /// 
 /// Debug logs are typically only recorded when the threshold is set to "debug"
 #[macro_export]
 macro_rules! log_debug {
     ($message:expr) => {
         $crate::Logger::debug($message, None, file!(), line!(), module_path!())
     };
     ($message:expr, $context:expr) => {
         $crate::Logger::debug($message, $context, file!(), line!(), module_path!())
     };
 }
 
 /// Log an info-level message
 /// 
 /// # Example
 /// ```
 /// log_info!("Application started successfully");
 /// log_info!("User profile updated", Some("profile_id=12345".to_string()));
 /// ```
 /// 
 /// Info logs are recorded when the threshold is "debug" or "info"
 #[macro_export]
 macro_rules! log_info {
     ($message:expr) => {
         $crate::Logger::info($message, None, file!(), line!(), module_path!())
     };
     ($message:expr, $context:expr) => {
         $crate::Logger::info($message, $context, file!(), line!(), module_path!())
     };
 }
 
 /// Log a warning-level message
 /// 
 /// # Example
 /// ```
 /// log_warn!("Database connection pool running low");
 /// log_warn!("API rate limit approaching", Some(format!("current_rate={}/sec", rate)));
 /// ```
 /// 
 /// Warning logs are recorded when the threshold is "debug", "info", or "warn"
 #[macro_export]
 macro_rules! log_warn {
     ($message:expr) => {
         $crate::Logger::warn($message, None, file!(), line!(), module_path!())
     };
     ($message:expr, $context:expr) => {
         $crate::Logger::warn($message, $context, file!(), line!(), module_path!())
     };
 }
 
 /// Log an error-level message
 /// 
 /// # Example
 /// ```
 /// log_error!("Failed to connect to database");
 /// log_error!("Payment processing failed", Some(format!("error_code={}", code)));
 /// ```
 /// 
 /// Error logs are always recorded regardless of threshold level
 #[macro_export]
 macro_rules! log_error {
     ($message:expr) => {
         $crate::Logger::error($message, None, file!(), line!(), module_path!())
     };
     ($message:expr, $context:expr) => {
         $crate::Logger::error($message, $context, file!(), line!(), module_path!())
     };
 }
 
 /// Ensures all pending log messages are processed before application exit
 /// 
 /// Call this function before your application terminates to ensure that
 /// asynchronous log messages in the channel are properly flushed.
 /// 
 /// # Returns
 /// - `Result<(), String>`: Success or error message
 /// 
 /// # Example
 /// ```
 /// fn main() {
 ///     // Initialize logger and application code...
 ///     
 ///     // Before exit, ensure logs are flushed
 ///     liblogger::shutdown_logger().unwrap_or_else(|e| {
 ///         eprintln!("Error during logger shutdown: {}", e);
 ///     });
 /// }
 /// ```
 pub fn shutdown_logger() -> Result<(), String> {
     Logger::shutdown()
 }
 