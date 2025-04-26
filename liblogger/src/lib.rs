/*
 * Main library entry point that exposes the public API
 * 
 * This file defines the public interface for the logging library, including:
 * - Re-exporting the Logger struct for initialization
 * - Re-exporting LogConfig and LogLevel for custom configuration
 * - Defining logging macros (log_debug, log_info, log_warn, log_error)
 * 
 * The macros provide a convenient way to log messages at different severity levels,
 * automatically capturing file, line, and module information.
 */

mod config;
mod outputs;
mod logger;

pub use logger::Logger;
pub use config::LogConfig;
pub use config::LogLevel;

// Add the macros to be publicly accessible
#[macro_export]
macro_rules! log_debug {
    ($message:expr) => {
        $crate::Logger::debug($message, None, file!(), line!(), module_path!())
    };
    ($message:expr, $context:expr) => {
        $crate::Logger::debug($message, $context, file!(), line!(), module_path!())
    };
}

#[macro_export]
macro_rules! log_info {
    ($message:expr) => {
        $crate::Logger::info($message, None, file!(), line!(), module_path!())
    };
    ($message:expr, $context:expr) => {
        $crate::Logger::info($message, $context, file!(), line!(), module_path!())
    };
}

#[macro_export]
macro_rules! log_warn {
    ($message:expr) => {
        $crate::Logger::warn($message, None, file!(), line!(), module_path!())
    };
    ($message:expr, $context:expr) => {
        $crate::Logger::warn($message, $context, file!(), line!(), module_path!())
    };
}

#[macro_export]
macro_rules! log_error {
    ($message:expr) => {
        $crate::Logger::error($message, None, file!(), line!(), module_path!())
    };
    ($message:expr, $context:expr) => {
        $crate::Logger::error($message, $context, file!(), line!(), module_path!())
    };
}
