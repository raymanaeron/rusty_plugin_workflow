// engine/src/main.rs

use plugin_api::{PluginApi, PluginContext};
use libloading::{Library, Symbol};
use std::ffi::CString;

fn main() {
    unsafe {
        let lib = Library::new("target/debug/plugin_wifi.dll").expect("Failed to load plugin");
        let constructor: Symbol<unsafe fn() -> *const PluginApi> = lib.get(b"create_plugin").unwrap();
        let api = constructor();

        let name_ptr = ((*api).name)();
        let name = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy();
        println!("Loaded plugin: {}", name);

        let config = CString::new("scan=true").unwrap();
        let ctx = PluginContext {
            config: config.as_ptr(),
        };

        ((*api).run)(&ctx);
    }
}
