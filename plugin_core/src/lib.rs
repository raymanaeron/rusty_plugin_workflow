use std::os::raw::{c_char, c_int, c_float};
use axum::Router;

#[repr(C)]
pub struct PluginContext {
    pub config: *const c_char,
}

#[repr(C)]
pub struct Plugin {
    pub name: extern "C" fn() -> *const c_char,
    pub run: extern "C" fn(ctx: *const PluginContext),
    
    // New additions
    pub get_api_route: extern "C" fn() -> *mut Router,
    pub get_static_content_route: extern "C" fn() -> *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub ssid: *const c_char,
    pub bssid: *const c_char,
    pub signal: c_int,
    pub channel: c_int,
    pub security: *const c_char,
    pub frequency: c_float,
}
