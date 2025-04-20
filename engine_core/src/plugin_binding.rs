use plugin_core::{ApiRequest, ApiResponse, Resource, PluginContext};

/// Represents a loaded plugin and the engine's active binding to it.
///
/// This structure is created by the engine after dynamically loading a plugin via FFI
/// and calling its `create_plugin()` function. It encapsulates all the information
/// and function pointers the engine needs to route REST requests and serve static content.
///
/// It does not belong to `plugin_core` because it is engine-specific — different
/// engines (e.g., headless, UI-driven) may construct and use it differently,
/// even if they all conform to the shared plugin interface.
pub struct PluginBinding {
    /// The unique name of the plugin, such as `"plugin_wifi"`, `"plugin_bluetooth"`, etc.
    pub name: String,
    
    /// This name is used to mount routes at:
    /// - `/wifi/api/<resource>` for REST APIs
    /// - `/wifi/web/<file>` for static web content
    pub plugin_route: String,

    /// The path to the plugin's static web assets folder.
    ///
    /// Returned from the plugin via `get_static_content_path()`. This folder
    /// is used to serve HTML, JavaScript, and other static files under `/web/`.
    pub static_path: String,

    /// Function pointer to retrieve a reference to the plugin's supported REST resources.
    ///
    /// Each resource defines:
    /// - A relative path like `"network"` or `"device/status"`
    /// - A list of supported HTTP methods for that resource
    ///
    /// This is typically backed by a static slice inside the plugin.
    pub get_api_resources: extern "C" fn(out_len: *mut usize) -> *const Resource,

    /// Function pointer to handle all plugin-level REST requests.
    ///
    /// The engine constructs an `ApiRequest` and passes it to this function.
    /// The plugin is responsible for returning a heap-allocated `ApiResponse`,
    /// which the engine will later free using `cleanup`.
    pub handle_request: extern "C" fn(request: *const ApiRequest) -> *mut ApiResponse,

    /// Function pointer used by the engine to deallocate memory returned in `ApiResponse`.
    ///
    /// The plugin is responsible for freeing:
    /// - `body_ptr` (e.g., Box<[u8]>)
    /// - `content_type` (CString)
    /// - Headers and their keys/values if present
    pub cleanup: extern "C" fn(response: *mut ApiResponse),

    /// Function pointer to run the plugin's main loop or event loop.
    pub run: extern "C" fn(ctx: *const PluginContext), 
}

impl Clone for PluginBinding {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            plugin_route: self.plugin_route.clone(),
            static_path: self.static_path.clone(),
            get_api_resources: self.get_api_resources,
            handle_request: self.handle_request,
            cleanup: self.cleanup,
            run: self.run,
        }
    }
}