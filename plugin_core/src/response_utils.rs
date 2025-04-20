use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use crate::ApiResponse;
use crate::HttpMethod;

pub fn json_response(status: u16, body: &str) -> *mut ApiResponse {
    let bytes = body.as_bytes().to_vec();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *const u8;
    let content_type = CString::new("application/json").unwrap().into_raw();

    let response = Box::new(ApiResponse {
        status,
        headers: ptr::null(),
        header_count: 0,
        content_type,
        body_ptr: ptr,
        body_len: len,
    });
    Box::into_raw(response)
}

pub fn text_response(status: u16, body: &str) -> *mut ApiResponse {
    let bytes = body.as_bytes().to_vec();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *const u8;
    let content_type = CString::new("text/plain").unwrap().into_raw();

    let response = Box::new(ApiResponse {
        status,
        headers: ptr::null(),
        header_count: 0,
        content_type,
        body_ptr: ptr,
        body_len: len,
    });
    Box::into_raw(response)
}

pub fn not_found_response() -> *mut ApiResponse {
    text_response(404, "Not Found")
}

pub fn method_not_allowed_response(method: HttpMethod, path: *const c_char) -> *mut ApiResponse {
    use std::ffi::CStr;
    let method_str = format!("{:?}", method);
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy() };
    let message = format!("Method {} not allowed on path '{}'", method_str, path_str);
    text_response(405, &message)
}