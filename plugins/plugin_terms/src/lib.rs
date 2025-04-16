use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::sync::Mutex;
use std::ptr;
use std::fs;

use plugin_core::{ApiRequest, ApiResponse, HttpMethod, Resource, Plugin, PluginContext};
use plugin_core::{method_not_allowed, cleanup_response};

static TERMS_PATH: &str = "userterms.txt";

// === Plugin Metadata ===

extern "C" fn name() -> *const c_char {
    CString::new("terms").unwrap().into_raw()
}

extern "C" fn run(ctx: *const PluginContext) {
    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config_cstr = CStr::from_ptr((*ctx).config);
        println!("Terms Plugin running with config: {}", config_cstr.to_string_lossy());
    }
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("terms/web").unwrap().into_raw()
}

// === API Resources ===

#[no_mangle]
pub extern "C" fn get_api_resources(count: *mut usize) -> *const Resource {
    static PATH: Mutex<Option<CString>> = Mutex::new(None);
    static METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];
    static RESOURCES: Mutex<Option<Box<[Resource]>>> = Mutex::new(None);

    let mut path_lock = PATH.lock().unwrap();
    let mut res_lock = RESOURCES.lock().unwrap();

    *path_lock = Some(CString::new("userterms").unwrap());

    *res_lock = Some(Box::new([Resource::new(
        path_lock.as_ref().unwrap().as_ptr(),
        METHODS.as_ptr(),
    )]));

    if !count.is_null() {
        unsafe { *count = res_lock.as_ref().unwrap().len(); }
    }

    res_lock.as_ref().unwrap().as_ptr()
}

// === API Handler ===

#[no_mangle]
pub extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
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
            HttpMethod::Get => {
                if path == "userterms" {
                    match fs::read_to_string(TERMS_PATH) {
                        Ok(content) => {
                            let bytes = content.into_bytes();
                            let body_len = bytes.len();
                            let body_ptr = Box::into_raw(bytes.into_boxed_slice()) as *const u8;
                            let content_type = CString::new("text/plain").unwrap().into_raw();

                            let response = Box::new(ApiResponse {
                                status: 200,
                                headers: ptr::null(),
                                header_count: 0,
                                content_type,
                                body_ptr,
                                body_len,
                            });

                            return Box::into_raw(response);
                        }
                        Err(err) => {
                            return plugin_core::error_response(500, &format!("Failed to read terms: {}", err));
                        }
                    }
                }
            }

            HttpMethod::Post => {
                if path == "userterms" {
                    println!("User accepted the terms.");
                    let msg = r#"{"message":"Terms accepted"}"#.as_bytes().to_vec();
                    let body_len = msg.len(); // Calculate the length before moving `msg`
                    let body_ptr = Box::into_raw(msg.into_boxed_slice()) as *const u8;
                    let content_type = CString::new("application/json").unwrap().into_raw();
            
                    let response = Box::new(ApiResponse {
                        status: 200,
                        headers: ptr::null(),
                        header_count: 0,
                        content_type,
                        body_ptr,
                        body_len, // Use the pre-calculated length
                    });
            
                    return Box::into_raw(response);
                }
            }
            _ => {
                let err_ptr = method_not_allowed(request.method, request.path);
                let body = CStr::from_ptr(err_ptr).to_bytes().to_vec();
                let len = body.len();
                let ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;
                let content_type = CString::new("text/plain").unwrap().into_raw();

                let response = Box::new(ApiResponse {
                    headers: ptr::null(),
                    header_count: 0,
                    content_type,
                    status: 405,
                    body_ptr: ptr,
                    body_len: len,
                });

                return Box::into_raw(response);
            }
        }

        // Default 404
        let body = "Resource not found".as_bytes().to_vec();
        let body_len = body.len(); // Calculate the length before moving `body`
        let ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;
        let content_type = CString::new("text/plain").unwrap().into_raw();

        let response = Box::new(ApiResponse {
            headers: ptr::null(),
            header_count: 0,
            content_type,
            status: 404,
            body_ptr: ptr,
            body_len, // Use the pre-calculated length
        });

        Box::into_raw(response)
    }
}

// === Cleanup + Plugin Export ===

#[no_mangle]
pub extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *const Plugin {
    &Plugin {
        name,
        run,
        get_static_content_path,
        get_api_resources,
        handle_request,
        cleanup,
    }
}
