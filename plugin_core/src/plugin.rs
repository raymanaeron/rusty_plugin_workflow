use std::os::raw::c_char;
use crate::ApiRequest;
use crate::ApiResponse;
use crate::PluginContext;
use crate::Resource;

/// Represents a dynamically loaded plugin and its exposed API to the engine.
///
/// Each plugin must implement this structure and return a pointer to it
/// via the `create_plugin()` symbol. The engine uses these methods to:
/// - Mount static UI assets
/// - Discover supported API endpoints
/// - Route and handle HTTP-style API requests
/// - Clean up memory returned by the plugin
#[repr(C)]
pub struct Plugin {
    /// Returns the name of the plugin (e.g., "WiFi Plugin").
    /// The returned value must be a null-terminated C string.
    pub name: extern "C" fn() -> *const c_char,

    /// Returns the route prefix for the plugin (e.g., "/wifi").
    /// This is used to mount the plugin's static content and API routes.
    pub plugin_route: extern "C" fn() -> *const c_char,

    /// Called once at plugin startup with configuration details.
    pub run: extern "C" fn(ctx: *const PluginContext),

    /// Returns the path to the plugin's static content folder.
    /// This will be mounted under `/<plugin>/` by the engine.
    pub get_static_content_path: extern "C" fn() -> *const c_char,

    /// Returns the list of API resources this plugin supports.
    /// The engine uses this list to build route handlers at `/<plugin>/api/<resource>`.
    ///
    /// # Parameters
    /// - `count`: Output parameter to receive the number of resources
    ///
    /// # Returns
    /// - Pointer to an array of `Resource` items. Must remain valid during plugin lifetime.
    //pub get_api_resources: extern "C" fn(count: *mut usize) -> *const Resource,
    pub get_api_resources: extern "C" fn(out_len: *mut usize) -> *const Resource,

    /// Handles an HTTP-style request dispatched to this plugin.
    /// The engine constructs an `ApiRequest` and passes it to the plugin.
    ///
    /// # Returns
    /// - A pointer to an `ApiResponse`, allocated by the plugin.
    /// - The engine will consume the response, then call `cleanup()` on it.
    pub handle_request: extern "C" fn(request: *const ApiRequest) -> *mut ApiResponse,

    /// Frees memory allocated for a plugin-generated `ApiResponse`.
    /// This is called by the engine after it finishes processing the response.
    ///
    /// The plugin must free all heap-allocated fields in `ApiResponse`, including:
    /// - `body_ptr`
    /// - `content_type`
    /// - Any allocated `Header` array (if used)
    ///
    /// # Example Implementation
    /// ```rust
    /// extern "C" fn cleanup(response: *mut ApiResponse) {
    ///     if response.is_null() {
    ///         return;
    ///     }
    ///     unsafe {
    ///         let r = Box::from_raw(response);
    ///         if !r.content_type.is_null() {
    ///             CString::from_raw(r.content_type as *mut c_char);
    ///         }
    ///         if !r.body_ptr.is_null() {
    ///             Vec::from_raw_parts(r.body_ptr as *mut u8, r.body_len, r.body_len);
    ///         }
    ///         if !r.headers.is_null() {
    ///             let headers = std::slice::from_raw_parts_mut(r.headers as *mut Header, r.header_count);
    ///             for h in headers {
    ///                 if !h.key.is_null() {
    ///                     CString::from_raw(h.key as *mut c_char);
    ///                 }
    ///                 if !h.value.is_null() {
    ///                     CString::from_raw(h.value as *mut c_char);
    ///                 }
    ///             }
    ///             Vec::from_raw_parts(r.headers as *mut Header, r.header_count, r.header_count);
    ///         }
    ///     }
    /// }
    /// ```
    pub cleanup: extern "C" fn(response: *mut ApiResponse),

    pub run_workflow: Option<extern "C" fn(input: *const ApiRequest) -> *mut ApiResponse>,
    pub on_progress: Option<extern "C" fn() -> *mut ApiResponse>,
    pub on_complete: Option<extern "C" fn() -> *mut ApiResponse>,
}