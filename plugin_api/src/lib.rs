// plugin_api/src/lib.rs

#[repr(C)]
pub struct PluginContext {
    pub config: *const libc::c_char,
}

#[repr(C)]
pub struct PluginApi {
    pub name: extern "C" fn() -> *const libc::c_char,
    pub run: extern "C" fn(ctx: *const PluginContext),
}
