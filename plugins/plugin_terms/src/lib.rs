use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::sync::Mutex;
use std::ptr;
use plugin_core::{ApiRequest, ApiResponse, HttpMethod, Resource, Plugin, PluginContext};
use plugin_core::{method_not_allowed, cleanup_response, error_response};
use logger::{LoggerLoader, LogLevel};

fn process_user_term_acceptance(accepted: bool) {
    if accepted {
        println!("[plugin_terms] user accepted the terms");
        let logger = LoggerLoader::get_logger();
        logger.log(LogLevel::Info, "user accepted the terms");
    } else {
        println!("[plugin_terms] user declined the terms");
        let logger = LoggerLoader::get_logger();
        logger.log(LogLevel::Warn, "user declined the terms");
    }
}

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

#[no_mangle]
pub extern "C" fn get_api_resources(count: *mut usize) -> *const Resource {
    static PATH: Mutex<Option<CString>> = Mutex::new(None);
    static METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];
    static RESOURCE_LIST: Mutex<Option<Box<[Resource]>>> = Mutex::new(None);

    let mut path_lock = PATH.lock().unwrap();
    let mut list_lock = RESOURCE_LIST.lock().unwrap();

    *path_lock = Some(CString::new("userterms").unwrap());

    *list_lock = Some(Box::new([Resource::new(
        path_lock.as_ref().unwrap().as_ptr(),
        METHODS.as_ptr(),
    )]));

    if !count.is_null() {
        unsafe {
            *count = list_lock.as_ref().unwrap().len();
        }
    }

    list_lock.as_ref().unwrap().as_ptr()
}

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
                    let json = r#"{ "terms": "Lorem empsum yada yada" }"#;
                    let bytes = json.as_bytes().to_vec();
                    let len = bytes.len();
                    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *const u8;
                    let content_type = CString::new("application/json").unwrap().into_raw();

                    let response = Box::new(ApiResponse {
                        status: 200,
                        headers: ptr::null(),
                        header_count: 0,
                        content_type,
                        body_ptr: ptr,
                        body_len: len,
                    });

                    return Box::into_raw(response);
                }
            }

            HttpMethod::Post => {
                if path == "userterms" {
                    let body_slice = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                    let body_str = std::str::from_utf8(body_slice).unwrap_or("");
                    let parsed: Result<serde_json::Value, _> = serde_json::from_str(body_str);

                    if let Ok(json) = parsed {
                        let accepted = json.get("accepted")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        process_user_term_acceptance(accepted);

                        let response_json = r#"{ "message": "Terms accepted" }"#;
                        let bytes = response_json.as_bytes().to_vec();
                        let len = bytes.len();
                        let ptr = Box::into_raw(bytes.into_boxed_slice()) as *const u8;
                        let content_type = CString::new("application/json").unwrap().into_raw();

                        let response = Box::new(ApiResponse {
                            status: 200,
                            headers: ptr::null(),
                            header_count: 0,
                            content_type,
                            body_ptr: ptr,
                            body_len: len,
                        });

                        return Box::into_raw(response);
                    } else {
                        return error_response(400, "Invalid JSON payload");
                    }
                }
            }

            _ => {
                let err_ptr = method_not_allowed(request.method, request.path);
                let body = CStr::from_ptr(err_ptr).to_bytes().to_vec();
                let len = body.len();
                let ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;
                let content_type = CString::new("text/plain").unwrap().into_raw();

                let response = Box::new(ApiResponse {
                    status: 405,
                    headers: ptr::null(),
                    header_count: 0,
                    content_type,
                    body_ptr: ptr,
                    body_len: len,
                });

                return Box::into_raw(response);
            }
        }

        // Fallback 404
        let msg = CString::new("Not Found").unwrap();
        let body = msg.as_bytes().to_vec();
        let len = body.len();
        let ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;
        let content_type = CString::new("text/plain").unwrap().into_raw();

        let response = Box::new(ApiResponse {
            status: 404,
            headers: ptr::null(),
            header_count: 0,
            content_type,
            body_ptr: ptr,
            body_len: len,
        });

        Box::into_raw(response)
    }
}

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
