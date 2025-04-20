extern crate plugin_core;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use plugin_core::*;
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;

use once_cell::sync::Lazy;

static STATUS: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("Ready.".to_string()));

#[ctor::ctor]
fn on_load() {
    println!("[plugin_status] >>> LOADED");
}

extern "C" fn run(_ctx: *const PluginContext) {
    println!("[plugin_status] - run");
    println!("[plugin_status] FINGERPRINT: run = {:p}", run as *const ());
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("status/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];
    let slice = static_resource("statusmessage", &METHODS);
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
            HttpMethod::Get if path == "statusmessage" => {
                let current = STATUS.lock().unwrap().clone();
                let json = format!(r#"{{ "status": "{}" }}"#, current);
                return json_response(200, &json);
            }

            HttpMethod::Post if path == "statusmessage" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                let body_str = std::str::from_utf8(body).unwrap_or("");
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(body_str);

                if let Ok(json) = parsed {
                    if let Some(status_str) = json.get("status").and_then(|v| v.as_str()) {
                        let mut shared = STATUS.lock().unwrap();
                        *shared = status_str.to_string();
                        return json_response(200, r#"{ "message": "Status updated" }"#);
                    }
                }

                return error_response(400, "Missing or invalid 'status' field in JSON payload");
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

declare_plugin!(
    "plugin_status",
    "status",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);
