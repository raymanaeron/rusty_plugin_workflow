//! Logging functionality for plugins
//!
//! This module re-exports logging macros from `liblogger` to make them available
//! to plugins through the plugin_core crate. This centralizes the logging dependency
//! and ensures all plugins use a consistent logging approach.

#[cfg(feature = "logging")]
pub use liblogger::{log_debug, log_info, log_warn, log_error, Logger};

#[cfg(feature = "logging")]
pub use liblogger_macros::*;

/// Initialize the logger for plugin usage
///
/// This function is designed to be called from a plugin's initialization routine.
/// It will attempt to use the same configuration as the engine by looking for 
/// app_config.toml in the current directory. If that fails, it will fall back
/// to console logging.
/// 
/// Note: The engine initializes the logger before loading plugins, so this function
/// will usually just connect to the already initialized logger rather than creating
/// a new instance.
///
/// # Arguments
///
/// * `plugin_name` - Name of the plugin (used for identification in logs)
///
/// # Returns
///
/// * `Result<(), String>` - Success or an error message
///
/// # Example
///
/// ```
/// use plugin_core::logging;
///
/// fn main() {
///     // Initialize logger at the start of your plugin
///     if let Err(e) = plugin_core::init_logger("my_plugin") {
///         eprintln!("Failed to initialize logger: {}", e);
///     }
/// }
/// ```
#[cfg(feature = "logging")]
pub fn init_logger(plugin_name: &str) -> Result<(), String> {
    // First try to initialize from app_config.toml
    match liblogger::Logger::init_with_config_file("app_config.toml") {
        Ok(_) => Ok(()),
        Err(e) => {
            // Log the error but continue with default console logger
            eprintln!("[{}] Error initializing logger from config: {}", plugin_name, e);
            liblogger::Logger::init();
            Ok(())
        }
    }
}

// Create no-op versions of the macros when logging is disabled
// These will be available at crate root level due to #[macro_export]
#[cfg(not(feature = "logging"))]
mod no_op_macros {
    /// No-op debug logging implementation
    #[macro_export]
    macro_rules! log_debug {
        ($($arg:tt)*) => {};
    }

    /// No-op info logging implementation
    #[macro_export]
    macro_rules! log_info {
        ($($arg:tt)*) => {};
    }

    /// No-op warning logging implementation  
    #[macro_export]
    macro_rules! log_warn {
        ($($arg:tt)*) => {};
    }

    /// No-op error logging implementation
    #[macro_export]
    macro_rules! log_error {
        ($($arg:tt)*) => {};
    }
}

#[cfg(not(feature = "logging"))]
pub fn init_logger(_plugin_name: &str) -> Result<(), String> {
    // No-op when logging is disabled
    Ok(())
}
