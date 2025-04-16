use std::ffi::CString;
use std::ptr;
use crate::{ApiResponse};

pub fn error_response(code: u16, msg: &str) -> *mut ApiResponse {
    let json = format!(r#"{{"message":"{}"}}"#, msg);
    let body = json.into_bytes();
    let body_len = body.len();
    let body_ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;

    let content_type = CString::new("application/json").unwrap();

    let response = Box::new(ApiResponse {
        status: code,
        headers: ptr::null(),
        header_count: 0,
        body_ptr,
        body_len,
        content_type: content_type.into_raw(),
    });

    Box::into_raw(response)
}
