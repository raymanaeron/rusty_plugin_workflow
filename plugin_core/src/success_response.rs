use crate::{ApiResponse, error_response};

pub fn success_response(body_json: &str) -> *mut ApiResponse {
    error_response(200, body_json)
}
