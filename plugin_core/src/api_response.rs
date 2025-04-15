use std::os::raw::c_char;
use crate::ApiHeader;

/// Represents an HTTP-style response returned from a plugin to the engine.
///
/// This structure allows a plugin to send back full HTTP metadata,
/// including status codes, content type, body data, and response headers.
///
/// The engine is responsible for converting this structure into an Axum `Response`.
#[repr(C)]
pub struct ApiResponse {
    /// Pointer to an array of response headers (key-value pairs).
    /// Used for setting custom headers like CORS, cookies, ETags, etc.
    pub headers: *const ApiHeader,

    /// Number of headers in the `headers` array.
    pub header_count: usize,

    /// The content type of the response body.
    /// Example: "application/json", "text/plain", "application/octet-stream"
    pub content_type: *const c_char,

    /// HTTP status code to return (e.g., 200 OK, 404 Not Found, 500 Internal Server Error).
    pub status: u16,

    /// Pointer to the raw response body bytes.
    /// Can contain serialized JSON, plain text, binary data, etc.
    pub body_ptr: *const u8,

    /// Length of the response body in bytes. Zero indicates an empty body.
    pub body_len: usize,
}