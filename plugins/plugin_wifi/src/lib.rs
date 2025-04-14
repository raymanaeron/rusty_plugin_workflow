// plugins/plugin_wifi/src/lib.rs

use plugin_api::{PluginApi, PluginContext};
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" fn name() -> *const c_char {
    CString::new("WiFi Plugin").unwrap().into_raw()
}

extern "C" fn run(ctx: *const PluginContext) {
    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config_cstr = std::ffi::CStr::from_ptr((*ctx).config);
        println!("WiFi Plugin running with config: {}", config_cstr.to_string_lossy());
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *const PluginApi {
    &PluginApi {
        name,
        run,
    }
}
