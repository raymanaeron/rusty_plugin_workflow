use std::os::raw::c_char;

/// Represents a single HTTP-style header as a key-value pair,
/// used in `ApiRequest` and `ApiResponse`.
///
/// This struct is FFI-safe and defines how custom headers are exchanged
/// between the engine and a plugin. It is not related to `http::Header` or `HeaderMap`
/// from other Rust libraries to avoid naming conflicts.
///
/// ### Memory Ownership
/// - The plugin must ensure that both `key` and `value` are valid, null-terminated UTF-8 C strings.
/// - If allocated dynamically, they must be freed by the plugin during `cleanup()`.
///
/// ### Examples
/// - `"Content-Type": "application/json"`
/// - `"Authorization": "Bearer token123"`
///
/// ### Usage
/// - `ApiRequest` receives headers from the engine.
/// - `ApiResponse` returns headers to the engine for the final response.
#[repr(C)]
pub struct ApiHeader {
    /// Header name (e.g., "Content-Type").
    pub key: *const c_char,

    /// Header value (e.g., "application/json").
    pub value: *const c_char,
}
