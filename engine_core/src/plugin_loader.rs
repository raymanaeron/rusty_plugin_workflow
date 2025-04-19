use std::ffi::CStr;
use std::path::Path;
use std::ptr;

use libloading::{Library, Symbol};
use plugin_core::Resource;
use crate::plugin_binding::PluginBinding;
use plugin_core::Plugin;

// Updated: now stores a real slice instead of raw pointer
static mut STATIC_RESOURCES: Option<&'static [Resource]> = None;

/// Loads a plugin from a shared library file and returns a PluginBinding.
/// This assumes the plugin exports a `create_plugin()` function.
pub fn load_plugin<P: AsRef<Path>>(path: P) -> Result<(PluginBinding, Library), String> {
    unsafe {
        println!("[engine] Loading plugin from: {:?}", path.as_ref().canonicalize());

        // Load the shared library
        let lib = Library::new(path.as_ref())
            .map_err(|e| format!("Failed to load plugin: {}", e))?;

        // Load the create_plugin symbol
        let constructor: Symbol<unsafe extern "C" fn() -> *const Plugin> =
            lib.get(b"create_plugin")
                .map_err(|e| format!("Missing symbol: {}", e))?;

        // Call create_plugin to get the plugin struct
        let plugin_ptr = constructor();
        if plugin_ptr.is_null() {
            return Err("Plugin returned null pointer".to_string());
        }

        let plugin = &*plugin_ptr;

        // Call plugin.name()
        let name_ptr = (plugin.name)();
        if name_ptr.is_null() {
            return Err("Plugin name() returned null".to_string());
        }

        let name_cstr = CStr::from_ptr(name_ptr);
        let name = name_cstr.to_string_lossy().into_owned();

        // Call plugin.get_static_content_path()
        let path_ptr = (plugin.get_static_content_path)();
        if path_ptr.is_null() {
            return Err("Plugin get_static_content_path() returned null".to_string());
        }

        let path_cstr = CStr::from_ptr(path_ptr);
        let static_path = path_cstr.to_string_lossy().into_owned();

        // Call plugin.get_api_resources correctly
        let mut count: usize = 0;
        let ptr = (plugin.get_api_resources)(&mut count);
        if ptr.is_null() || count == 0 {
            return Err(format!("Plugin {} returned no resources", name));
        }

        let resource_slice = std::slice::from_raw_parts(ptr, count);
        STATIC_RESOURCES = Some(resource_slice);

        // Construct the PluginBinding
        let binding = PluginBinding {
            name,
            static_path,
            get_api_resources: plugin.get_api_resources,
            handle_request: plugin.handle_request,
            cleanup: plugin.cleanup,
            run: plugin.run,
        };

        Ok((binding, lib))
    }
}

#[allow(dead_code)]
/// Returns a pointer to static resources and writes count to `out_len`
fn get_static_resources(out_len: *mut usize) -> *const Resource {
    unsafe {
        if let Some(slice) = STATIC_RESOURCES {
            if !out_len.is_null() {
                *out_len = slice.len();
            }
            slice.as_ptr()
        } else {
            ptr::null()
        }
    }
}
