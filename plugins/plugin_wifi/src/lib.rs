//! WiFi Plugin Module
//! 
//! This module provides WiFi scanning and connection capabilities as a plugin.
//! It supports multiple platforms (Windows, Linux, macOS) and handles network
//! discovery and connection management.

extern crate plugin_core;

mod network_info;
pub mod wifi_manager_cp;

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

use plugin_core::*;
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;

use once_cell::sync::Lazy;

use network_info::{ NetworkInfo, to_json };

/// Global flag to track WiFi connection status
static WIFI_CONNECTED: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

/// Plugin initialization handler
/// Called when the plugin is first loaded
#[ctor::ctor]
fn on_load() {
    println!("[plugin_wifi] >>> LOADED");
}

/// Plugin runtime configuration handler
/// 
/// # Arguments
/// * `ctx` - Pointer to plugin context containing configuration
extern "C" fn run(ctx: *const PluginContext) {
    println!("[plugin_wifi] - run");
    println!("[plugin_wifi] FINGERPRINT: run = {:p}", run as *const ());

    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config_cstr = CStr::from_ptr((*ctx).config);
        println!("WiFi Plugin running with config: {}", config_cstr.to_string_lossy());
    }
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("wifi/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];
    let slice = static_resource("network", &METHODS);
    unsafe { *out_len = slice.len(); }
    slice.as_ptr()
}

extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let request = &*req;
        let path = if request.path.is_null() {
            "<null>"
        } else {
            CStr::from_ptr(request.path).to_str().unwrap_or("<invalid>")
        };

        match request.method {
            HttpMethod::Get if path == "network" => {
                let mut count: usize = 0;
                let result_ptr = scan(&mut count);

                let json = if result_ptr.is_null() || count == 0 {
                    "[]".to_string()
                } else {
                    let results = std::slice::from_raw_parts(result_ptr, count);
                    let objects: Vec<_> = results.iter().map(to_json).collect();
                    serde_json::to_string(&objects).unwrap_or("[]".into())
                };

                return json_response(200, &json);
            }

            HttpMethod::Post if path == "network" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                let body_str = std::str::from_utf8(body).unwrap_or("");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(body_str) {
                    let ssid = json.get("ssid").and_then(|v| v.as_str()).unwrap_or("");
                    let password = json.get("password").and_then(|v| v.as_str()).unwrap_or("");
                    return connect_to_network(ssid, password);
                }
                return error_response(400, "Invalid JSON payload");
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

extern "C" fn scan(out_count: *mut usize) -> *mut NetworkInfo {
    wifi_manager_cp::scan(out_count)
}

fn connect_to_network(ssid: &str, password: &str) -> *mut ApiResponse {
    let success = wifi_manager_cp::connect_wifi(ssid, password);
    
    {
        let mut flag = WIFI_CONNECTED.lock().unwrap();
        *flag = success;
    }

    if success {
        println!("[plugin_wifi] Connected to {}", ssid);
        let msg = format!(r#"{{ "message": "Connected to {}" }}"#, ssid);
        json_response(200, &msg)
    } else {
        let msg = format!(r#"{{ "message": "Failed to connect to {}" }}"#, ssid);
        json_response(500, &msg)
    }
}

extern "C" fn on_complete() -> *mut ApiResponse {
    let connected = *WIFI_CONNECTED.lock().unwrap();
    println!("[plugin_wifi] on_complete: connected = {}", connected);

    if connected {
        json_response(200, r#"{ "message": "WiFi Connected" }"#)
    } else {
        json_response(204, r#"{ "message": "WiFi not connected" }"#)
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

extern "C" fn null_workflow(_req: *const ApiRequest) -> *mut ApiResponse {
    std::ptr::null_mut()
}

extern "C" fn null_progress() -> *mut ApiResponse {
    std::ptr::null_mut()
}

declare_plugin!(
    "plugin_wifi",
    "wifi",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup,
    null_workflow,
    null_progress,
    on_complete
);
