/// Represents the supported HTTP methods for a plugin-exposed API resource.
///
/// This enum is used in both `ApiRequest` (to indicate the incoming method)
/// and in `Resource` (as a bitmask to declare which methods are supported).
///
/// Each variant maps to its standard HTTP counterpart and is encoded as a `u8`
/// to ensure compatibility across the FFI boundary.
///
/// ### Usage in `Resource`
/// Plugins declare supported methods using a bitmask:
/// - GET    → 0b0001 = 1
/// - POST   → 0b0010 = 2
/// - PUT    → 0b0100 = 4
/// - DELETE → 0b1000 = 8
///
/// So a plugin that supports GET and POST on a resource would declare:
/// `supported_methods = HttpMethod::Get as u8 | HttpMethod::Post as u8`
///
/// ### Usage in `ApiRequest`
/// The engine populates the `method` field with the appropriate variant.
///
/// ### FFI-Safety
/// This enum must always be used with `#[repr(u8)]` to ensure that it
/// maps to a predictable, compact layout across plugin boundaries.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    /// HTTP GET: used to retrieve resource data (safe and idempotent).
    Get = 0,

    /// HTTP POST: used to create or submit data to a resource.
    Post = 1,

    /// HTTP PUT: used to update a resource entirely.
    Put = 2,

    /// HTTP DELETE: used to remove a resource.
    Delete = 3,
}

impl PartialEq for HttpMethod {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (HttpMethod::Get, HttpMethod::Get)
                | (HttpMethod::Post, HttpMethod::Post)
                | (HttpMethod::Put, HttpMethod::Put)
                | (HttpMethod::Delete, HttpMethod::Delete)
        )
    }
}