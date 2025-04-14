use std::sync::Arc;
use libloading::{Library, Symbol};
use plugin_api::PluginApi;

pub struct LoadedPlugin {
    pub _lib: Library,
    pub api: Arc<PluginApi>,
}

pub fn load_plugin(path: &str) -> Result<LoadedPlugin, String> {
    unsafe {
        let lib = Library::new(path).map_err(|e| format!("Failed to load plugin: {}", e))?;
        let constructor: Symbol<unsafe extern "C" fn() -> *const PluginApi> =
            lib.get(b"create_plugin").map_err(|e| format!("Symbol error: {}", e))?;

        let raw = constructor();
        if raw.is_null() {
            return Err("Plugin constructor returned null".to_string());
        }

        Ok(LoadedPlugin {
            _lib: lib,
            api: Arc::from_raw(raw),
        })
    }
}
