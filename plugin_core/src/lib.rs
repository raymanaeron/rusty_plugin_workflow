pub mod api_request;
pub mod api_response;
pub mod api_header;
pub mod http_method;
pub mod plugin_context;
pub mod plugin;
pub mod resource;
pub mod helper_functions;
pub mod logging;

pub mod response_utils;
pub mod resource_utils;

#[macro_use]
mod plugin_macros;

pub use api_request::ApiRequest;
pub use api_response::ApiResponse;
pub use api_header::ApiHeader;
pub use http_method::HttpMethod;
pub use plugin_context::PluginContext;
pub use plugin::Plugin;
pub use resource::Resource;

pub use helper_functions::error_response;
pub use helper_functions::success_response;
pub use helper_functions::method_not_allowed;
pub use helper_functions::cleanup_response;

// When logging feature is enabled, re-export from liblogger
#[cfg(feature = "logging")]
pub use liblogger::{log_debug, log_info, log_warn, log_error};

// Re-export initialize_logger_attributes! macro from liblogger_macros when logging feature is enabled
#[cfg(feature = "logging")]
pub use liblogger_macros::initialize_logger_attributes;

// When logging is disabled, the no-op macros are automatically exported
// at crate root level by #[macro_export], so we don't need to re-export them

// Always export the init_logger function
pub use logging::init_logger;

// Add a convenience function to check if logging is enabled
/// Returns true if the logging feature is enabled
pub fn logging_enabled() -> bool {
    cfg!(feature = "logging")
}