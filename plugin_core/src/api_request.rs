use std::os::raw::c_char;
use crate::HttpMethod;
use crate::ApiHeader;

/// Represents an HTTP-style request sent from the engine to a plugin.
///
/// This structure is designed to be flexible and FFI-safe, allowing plugins
/// to handle a variety of API requests. It supports raw byte bodies and headers,
/// enabling full compatibility with JSON, binary data, and structured metadata.
///
/// All strings (`path`, `query`, `content_type`) must be valid null-terminated C strings (UTF-8).
///
/// NOTE: We use raw `*const u8` + `len` for the body instead of a `JSON` type,
/// because the body may contain arbitrary binary data, not just UTF-8 strings.
/// This makes the system extensible for future content types like file uploads
/// or binary protocols, even if most plugins currently use JSON.
#[repr(C)]
pub struct ApiRequest {
    /// The resource path relative to the plugin's `/api` base.
    /// For example: "network", "device/status", etc.
    pub path: *const c_char,

    /// The HTTP method of the request (GET, POST, PUT, DELETE).
    pub method: HttpMethod,

    /// Pointer to an array of key-value header pairs.
    /// Each key and value must be null-terminated C strings.
    pub headers: *const ApiHeader,

    /// The content type of the request body. For example: "application/json".
    /// May be null or empty for GET requests or empty bodies.
    pub content_type: *const c_char,

    /// Number of headers in the `headers` array.
    pub header_count: usize,

    /// Optional query string, without the leading '?'. For example: "page=1&sort=asc".
    /// Null or empty string if not present.
    pub query: *const c_char,

    /// Pointer to the raw body bytes. This allows the engine to forward binary,
    /// JSON, or other encodings as-is without interpretation.
    pub body_ptr: *const u8,

    /// Length of the body in bytes. Zero indicates an empty body.
    pub body_len: usize,
}