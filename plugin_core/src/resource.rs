use std::os::raw::c_char;
use crate::HttpMethod;
use std::marker::PhantomData;

/// Describes a single API resource exposed by a plugin.
///
/// Each resource represents a REST-style endpoint that the engine should mount under the plugin’s
/// base path (typically `/api`). The plugin uses this structure to declare all supported
/// routes and the HTTP methods that apply to each one.
///
#[repr(C)]
#[derive(Debug)]
pub struct Resource {
    /// The relative path for the resource, such as "network" or "device/status".
    /// This must be a null-terminated UTF-8 string.
    pub path: *const c_char,

    /// A pointer to a list of supported HTTP methods for this resource.
    pub supported_methods: *const HttpMethod,

    /// Marker to indicate raw pointers are not Send
    _marker: PhantomData<*const ()>,
}

impl Resource {
    pub fn new(path: *const c_char, supported_methods: *const HttpMethod) -> Self {
        Self {
            path,
            supported_methods,
            _marker: PhantomData,
        }
    }
}

// Manually implement Send for Resource
unsafe impl Send for Resource {}

impl Clone for Resource {
    fn clone(&self) -> Self {
        Self {
            path: self.path,
            supported_methods: self.supported_methods,
            _marker: PhantomData,
        }
    }
}