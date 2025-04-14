use std::os::raw::c_char;

#[repr(C)]
pub struct PluginContext {
    pub config: *const c_char,
}

#[repr(C)]
pub struct PluginApi {
    pub name: extern "C" fn() -> *const c_char,
    pub run: extern "C" fn(ctx: *const PluginContext),
}
