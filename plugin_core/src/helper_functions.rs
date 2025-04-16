use std::ffi::{ CString, c_char };
use std::ptr;
use crate::{ApiHeader, ApiResponse, HttpMethod};

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

pub fn success_response(body_json: &str) -> *mut ApiResponse {
    error_response(200, body_json)
}

pub fn method_not_allowed(method: HttpMethod, resource: *const c_char) -> *const c_char {
    let method_str = match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
    };

    let res_str = unsafe {
        if resource.is_null() {
            "<unknown>"
        } else {
            std::ffi::CStr::from_ptr(resource).to_str().unwrap_or("<invalid>")
        }
    };

    let msg = format!("Method {} not allowed on {}", method_str, res_str);
    CString::new(msg).unwrap().into_raw()
}

pub fn cleanup_response(response: *mut ApiResponse) {
    if response.is_null() {
        return;
    }

    unsafe {
        let resp = Box::from_raw(response);

        if !resp.body_ptr.is_null() && resp.body_len > 0 {
            let body_slice = std::slice::from_raw_parts_mut(resp.body_ptr as *mut u8, resp.body_len);
            let _ = Box::from_raw(body_slice as *mut [u8]);
        }

        if !resp.content_type.is_null() {
            let _ = CString::from_raw(resp.content_type as *mut c_char);
        }

        if !resp.headers.is_null() && resp.header_count > 0 {
            let headers_slice = std::slice::from_raw_parts_mut(resp.headers as *mut ApiHeader, resp.header_count);
            for header in &mut *headers_slice {
                if !header.key.is_null() {
                    let _ = CString::from_raw(header.key as *mut c_char);
                }
                if !header.value.is_null() {
                    let _ = CString::from_raw(header.value as *mut c_char);
                }
            }
            let _ = Box::from_raw(headers_slice as *mut [ApiHeader]);
        }
    }
}