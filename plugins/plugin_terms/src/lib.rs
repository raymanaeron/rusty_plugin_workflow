extern crate plugin_core;

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;

use plugin_core::*;
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;
use plugin_core::jwt_utils::validate_jwt_token;

#[ctor::ctor]
fn on_load() {
    println!("[plugin_terms] >>> LOADED");
}

extern "C" fn run(ctx: *const PluginContext) {
    println!("[plugin_terms] - run");
    println!("[plugin_terms] FINGERPRINT: run = {:p}", run as *const ());

    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config = CStr::from_ptr((*ctx).config);
        println!("Terms Plugin running with config: {}", config.to_string_lossy());
    }
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("terms/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];
    let slice = static_resource("userterms", &METHODS);
    unsafe { *out_len = slice.len(); }
    slice.as_ptr()
}

extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
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

        println!("[plugin_terms] Received {:?} on path = '{}'", request.method, path);

        match request.method {
            HttpMethod::Get if path == "userterms" => {
                return json_response(200, r#"{ "terms": "Lorem empsum yada yada" }"#);
            }

            HttpMethod::Post if path == "userterms" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                let body_str = std::str::from_utf8(body).unwrap_or("");
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(body_str);

                if let Ok(json) = parsed {
                    let accepted = json.get("accepted").and_then(|v| v.as_bool()).unwrap_or(false);
                    process_user_term_acceptance(accepted);
                    return json_response(200, if accepted {
                        r#"{ "message": "Terms accepted" }"#
                    } else {
                        r#"{ "message": "Terms declined" }"#
                    });
                }

                return error_response(400, "Invalid JSON payload");
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

fn process_user_term_acceptance(accepted: bool) {
    if accepted {
        println!("[plugin_terms] user accepted the terms");
    } else {
        println!("[plugin_terms] user declined the terms");
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

declare_plugin!(
    "plugin_terms",
    "terms",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);
