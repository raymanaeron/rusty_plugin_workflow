use std::ffi::CStr;
use std::path::Path;

use libloading::{Library, Symbol};
use plugin_core::Resource;
use crate::plugin_binding::PluginBinding;
use plugin_core::Plugin;

static mut STATIC_RESOURCES: Option<&'static [Resource]> = None;

/// Loads a plugin from a shared library file and returns a PluginBinding.
/// This assumes the plugin exports a `create_plugin()` function.
pub fn load_plugin<P: AsRef<Path>>(path: P) -> Result<(PluginBinding, Library), String> {
    unsafe {
        // Load the shared library
        let lib = Library::new(path.as_ref()).map_err(|e| format!("Failed to load plugin: {}", e))?;

        // Load the create_plugin symbol
        let constructor: Symbol<unsafe extern "C" fn() -> *const Plugin> =
            lib.get(b"create_plugin").map_err(|e| format!("Missing symbol: {}", e))?;

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

        // Call plugin.get_api_resources() and copy to static slice
        let mut count: usize = 0;
        let resource_ptr = (plugin.get_api_resources)(&mut count as *mut usize);
        if resource_ptr.is_null() || count == 0 {
            return Err(format!("Plugin {} returned no resources", name));
        }

        let resource_slice = std::slice::from_raw_parts(resource_ptr, count);

        // Copy resources into a static buffer
        let copied_resources: Box<[Resource]> = resource_slice.into();
        STATIC_RESOURCES = Some(Box::leak(copied_resources)); // Store in static variable

        // Construct and return the PluginBinding
        let binding = PluginBinding {
            name,
            static_path,
            get_supported_resources: get_static_resources, // Use the function pointer
            handle_request: plugin.handle_request,
            cleanup: plugin.cleanup,
            run: plugin.run,
        };

        // Return both the binding and the Library to keep it alive
        Ok((binding, lib))
    }
}

fn get_static_resources() -> &'static [Resource] {
    unsafe {
        STATIC_RESOURCES.unwrap() // Access the static variable
    }
}