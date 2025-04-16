use std::ffi::CString;
use std::ptr;
use crate::{ApiResponse};

pub fn success_response(body_json: &str) -> *mut ApiResponse {
    error_response(200, body_json)
}
