//! JWT Utilities Module
//! 
//! This module provides JWT validation utilities for plugin authentication.

use std::ffi::CStr;
use crate::{ApiRequest, ApiResponse, error_response};
use crate::log_debug;
use crate::log_warn;

/// Validates JWT token from an API request
///
/// This function extracts the Authorization header from the request,
/// validates the JWT token format, and verifies the token using libjwt.
///
/// # Arguments
/// * `request` - API request containing headers
///
/// # Returns
/// * `Ok(())` - If token is valid
/// * `Err(*mut ApiResponse)` - If token is invalid or missing, contains error response
pub fn validate_jwt_token(request: &ApiRequest) -> Result<(), *mut ApiResponse> {
    // Extract authorization header from request if it exists
    let auth_header = if !request.headers.is_null() && request.header_count > 0 {
        let headers = unsafe { std::slice::from_raw_parts(request.headers, request.header_count as usize) };
        
        // Search for Authorization header
        headers.iter().find_map(|header| {
            let name = unsafe { CStr::from_ptr(header.key).to_str().ok()? };
            if name.eq_ignore_ascii_case("Authorization") {
                unsafe { CStr::from_ptr(header.value).to_str().ok() }
            } else {
                None
            }
        })
    } else {
        None
    };
    
    // Validate JWT token if authorization header exists
    if let Some(auth) = auth_header {
        if !auth.starts_with("Bearer ") {
            log_warn!("Invalid Authorization format, expected Bearer token");
            return Err(error_response(401, "Invalid Authorization format, expected Bearer token"));
        }
        
        let token = &auth[7..]; // Skip "Bearer " prefix
        
        match libjwt::validate_jwt(token) {
            Ok(claims) => {
                // Token is valid, continue with request processing
                log_debug!("JWT token validation successful");
                log_debug!(format!("Claims: {:?}", claims).as_str());
                Ok(())
            }
            Err(e) => {
                log_warn!(format!("JWT validation failed: {}", e).as_str());
                Err(error_response(401, "Invalid or expired token"))
            }
        }
    } else {
        log_warn!("No Authorization header found");
        Err(error_response(401, "Authentication required"))
    }
}
