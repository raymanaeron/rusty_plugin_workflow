//! WiFi Plugin Module
//! 
//! This module provides WiFi scanning and connection capabilities as a plugin.
//! It supports multiple platforms (Windows, Linux, macOS) and handles network
//! discovery and connection management.

// External crates
extern crate liblogger;
extern crate plugin_core;
extern crate liblogger_macros;
extern crate libjwt;

use liblogger_macros::{log_entry_exit, measure_time};
use once_cell::sync::Lazy;

use plugin_core::{
    log_debug, log_info, log_warn, log_error,
    declare_plugin, PluginContext, Resource, HttpMethod,
    ApiRequest, ApiResponse, error_response, cleanup_response
};
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;
use plugin_core::jwt_utils::validate_jwt_token;

// Standard library
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

// Add JWT validation from libjwt
use libjwt::validate_jwt;

// Internal modules
mod network_info;
pub mod wifi_manager_cp;
use network_info::{NetworkInfo, to_json};

// Initialize logger attributes
liblogger_macros::initialize_logger_attributes!();

/// Global flag to track WiFi connection status
static WIFI_CONNECTED: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

/// Plugin initialization handler
/// Called when the plugin is first loaded
#[ctor::ctor]
fn on_load() {
    // Initialize the logger for this plugin
    if let Err(e) = plugin_core::init_logger("plugin_wifi") {
        eprintln!("[plugin_wifi] Failed to initialize logger: {}", e);
    }
    
    log_info!("Plugin WiFi loaded successfully");
}

/// Plugin runtime configuration handler
/// 
/// # Arguments
/// * `ctx` - Pointer to plugin context containing configuration
#[log_entry_exit]
extern "C" fn run(ctx: *const PluginContext) {
    log_info!("Starting WiFi plugin initialization");

    if ctx.is_null() {
        log_error!("PluginContext is null");
        return;
    }

    unsafe {
        let config_cstr = CStr::from_ptr((*ctx).config);
        log_debug!(format!("WiFi Plugin running with config: {}", config_cstr.to_string_lossy()).as_str());
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

#[measure_time]
extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
        log_warn!("Received null request pointer");
        return ptr::null_mut();
    }

    unsafe {
        let request = &*req;
        
        // Validate JWT token using the shared utility function
        if let Err(response) = validate_jwt_token(request) {
            return response;
        }
        
        let path = if request.path.is_null() {
            "<null>"
        } else {
            CStr::from_ptr(request.path).to_str().unwrap_or("<invalid>")
        };

        log_debug!(format!("Handling API request path={}, method={:?}", path, request.method).as_str());

        match request.method {
            HttpMethod::Get if path == "network" => {
                log_info!("Processing network scan request");
                let mut count: usize = 0;
                let result_ptr = scan(&mut count);

                let json = if result_ptr.is_null() || count == 0 {
                    log_warn!("Scan returned no networks");
                    "[]".to_string()
                } else {
                    let results = std::slice::from_raw_parts(result_ptr, count);
                    let objects: Vec<_> = results.iter().map(to_json).collect();
                    log_info!(format!("Scan completed successfully, found_networks={}", count).as_str());
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
                    log_info!(format!("Processing connection request for ssid={}", ssid).as_str());
                    return connect_to_network(ssid, password);
                }
                log_error!("Invalid JSON in connection request");
                return error_response(400, "Invalid JSON payload");
            }

            _ => {
                log_warn!(format!("Method not allowed: method={:?}, path={}", request.method, path).as_str());
                method_not_allowed_response(request.method, request.path)
            },
        }
    }
}

extern "C" fn scan(out_count: *mut usize) -> *mut NetworkInfo {
    log_info!("Starting WiFi network scan");
    wifi_manager_cp::scan(out_count)
}

#[measure_time]
fn connect_to_network(ssid: &str, password: &str) -> *mut ApiResponse {
    log_info!(format!("Attempting to connect to network ssid={}", ssid).as_str());
    
    let success = wifi_manager_cp::connect_wifi(ssid, password);
    
    {
        let mut flag = WIFI_CONNECTED.lock().unwrap();
        *flag = success;
    }

    if success {
        log_info!(format!("Successfully connected to WiFi network ssid={}", ssid).as_str());
        let msg = format!(r#"{{ "message": "Connected to {}" }}"#, ssid);
        json_response(200, &msg)
    } else {
        log_error!(format!("Failed to connect to WiFi network ssid={}", ssid).as_str());
        let msg = format!(r#"{{ "message": "Failed to connect to {}" }}"#, ssid);
        json_response(500, &msg)
    }
}

extern "C" fn on_complete() -> *mut ApiResponse {
    let connected = *WIFI_CONNECTED.lock().unwrap();
    log_debug!(format!("on_complete: connected = {}", connected).as_str());

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
