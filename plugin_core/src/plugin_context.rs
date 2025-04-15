use std::os::raw::c_char;

/// Represents runtime context passed from the engine to a plugin at initialization.
///
/// This structure allows the engine to provide optional configuration data to the plugin
/// during startup. It can be used to supply static settings, feature flags, or step-specific
/// metadata that influence plugin behavior.
///
/// ### Usage
/// The engine calls the plugin's `run()` method with a pointer to this struct.
///
/// ### Example Contents of `config`
/// The `config` field is a null-terminated UTF-8 string. It may contain:
/// - Simple key-value pairs: `"scan=true;timeout=3000"`
/// - JSON strings: `"{\"scan\":true,\"timeout\":3000}"`
/// - Plugin-specific syntax
///
/// It is up to the plugin to parse this string appropriately.
///
/// ### Safety
/// - The `config` pointer must be either null or point to a valid null-terminated C string.
/// - The plugin must not modify or deallocate the memory behind `config`.
#[repr(C)]
pub struct PluginContext {
    /// Optional configuration string passed to the plugin at startup.
    /// This is a null-terminated UTF-8 C string. May be null.
    pub config: *const c_char,
}