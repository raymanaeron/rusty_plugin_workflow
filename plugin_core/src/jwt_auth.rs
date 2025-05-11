// JWT Authentication Module for Plugin Core
//
// This module provides centralized JWT token validation for plugins
// by leveraging functionality from the libjwt crate.

use crate::{
    ApiRequest, ApiResponse,
    error_response,
    log_warn, log_info, log_debug,
};
use libjwt::{
    validate_jwt,
    SharedTokenCache,
    models::TokenCacheEntry,
};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::ffi::CStr;
use std::time::Instant;

// Global shared token cache that all plugins can use
static TOKEN_CACHE: Lazy<SharedTokenCache> = Lazy::new(|| {
    log_debug!("Initializing shared token cache for JWT authentication");
    Arc::new(Mutex::new(std::collections::HashMap::new()))
});

/// Validates the JWT token from a request
///
/// # Arguments
///
/// * `request` - Reference to ApiRequest containing headers and other request data
///
/// # Returns
///
/// * `Option<*mut ApiResponse>` - Some(error_response) if validation fails, None if successful
pub fn validate_request_auth(request: &ApiRequest) -> Option<*mut ApiResponse> {
    log_debug!("Validating JWT token");
    
    // Find the Authorization header in the request
    let auth_header = match find_auth_header(request) {
        Some(header) => header,
        None => {
            log_warn!("Missing Authorization header");
            return Some(error_response(401, "Authentication required"));
        }
    };
    
    // Validate token format and extract token
    if !auth_header.starts_with("Bearer ") {
        log_warn!("Authorization header must use Bearer scheme");
        return Some(error_response(401, "Authorization header must use Bearer scheme"));
    }
    
    let token = &auth_header[7..]; // Skip "Bearer "
    
    // Validate the JWT token using libjwt's validate_jwt
    match validate_jwt(token) {
        Ok(claims) => {
            let cache = TOKEN_CACHE.lock().unwrap();
            let cache_key = format!("{}:{}", claims.sub, claims.session_id);
            
            if cache.contains_key(&cache_key) {
                log_info!("JWT token validated successfully");
                None // No error, authentication successful
            } else {
                log_warn!("Invalid session in JWT token");
                Some(error_response(401, "Invalid session"))
            }
        }
        Err(e) => {
            log_warn!(format!("Invalid JWT token: {}", e).as_str());
            Some(error_response(401, &format!("Invalid JWT token: {}", e)))
        }
    }
}

/// Finds the Authorization header in the request headers
///
/// # Arguments
///
/// * `request` - Reference to ApiRequest
///
/// # Returns
///
/// * `Option<String>` - Some(header_value) if found, None otherwise
fn find_auth_header(request: &ApiRequest) -> Option<String> {
    if request.headers.is_null() || request.header_count == 0 {
        log_warn!("No headers in request");
        return None;
    }
    
    unsafe {
        let headers = std::slice::from_raw_parts(request.headers, request.header_count);
        
        for header in headers {
            let key = CStr::from_ptr(header.key).to_string_lossy();
            if key.eq_ignore_ascii_case("Authorization") {
                let value = CStr::from_ptr(header.value).to_string_lossy().to_string();
                return Some(value);
            }
        }
    }
    
    log_warn!("No Authorization header found in request");
    None
}

/// Adds a token to the shared token cache
///
/// # Arguments
///
/// * `api_key` - API key for the user
/// * `session_id` - Session ID for the token
/// * `token` - The JWT token string
/// * `api_secret` - Optional API secret
pub fn add_token_to_cache(api_key: &str, session_id: &str, token: &str, api_secret: &str) {
    let cache_key = format!("{}:{}", api_key, session_id);
    let now = Instant::now();
    
    let entry = TokenCacheEntry {
        api_key: api_key.to_string(),
        api_secret: api_secret.to_string(),
        session_id: session_id.to_string(),
        token: token.to_string(),
        created_at: now,
        last_renewed: now,
    };
    
    let mut cache = TOKEN_CACHE.lock().unwrap();
    cache.insert(cache_key, entry);
    log_debug!("Added token to cache");
}

/// Removes a token from the shared token cache
///
/// # Arguments
///
/// * `api_key` - API key for the user
/// * `session_id` - Session ID for the token
pub fn remove_token_from_cache(api_key: &str, session_id: &str) {
    let cache_key = format!("{}:{}", api_key, session_id);
    let mut cache = TOKEN_CACHE.lock().unwrap();
    cache.remove(&cache_key);
    log_debug!("Removed token from cache");
}

/// Gets a reference to the shared token cache
pub fn get_shared_token_cache() -> &'static SharedTokenCache {
    &TOKEN_CACHE
}