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

// Global token cache using a different approach to avoid re-initialization
// We use a static reference with Once to ensure it's only initialized once
static TOKEN_CACHE: Lazy<Arc<Mutex<Option<SharedTokenCache>>>> = Lazy::new(|| {
    // This is only called once when TOKEN_CACHE is first accessed
    log_debug!("Initializing global JWT token cache");
    Arc::new(Mutex::new(None))
});

// Global flag to track if the warning has been logged
static LOGGED_WARNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
            // Access the static token cache
            let token_cache_guard = TOKEN_CACHE.lock().unwrap();
            
            match &*token_cache_guard {
                Some(token_cache) => {
                    let cache = token_cache.lock().unwrap();
                    let cache_key = format!("{}:{}", claims.sub, claims.session_id);
                    
                    if cache.contains_key(&cache_key) {
                        log_info!("JWT token validated successfully");
                        None // No error, authentication successful
                    } else {
                        log_warn!("Invalid session in JWT token");
                        Some(error_response(401, "Invalid session"))
                    }
                }
                None => {
                    // Only log this warning once to avoid spamming logs
                    if !LOGGED_WARNING.swap(true, std::sync::atomic::Ordering::SeqCst) {
                        log_warn!("JWT token cache not initialized in plugin - this will only be logged once");
                    }
                    Some(error_response(401, "JWT authentication not configured"))
                }
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
    
    let token_cache_guard = TOKEN_CACHE.lock().unwrap();
    
    // If the cache is not initialized, log a warning and return
    if let Some(token_cache) = &*token_cache_guard {
        let mut token_cache = token_cache.lock().unwrap();
        token_cache.insert(cache_key, entry);
        log_debug!("Added token to cache");
    } else {
        log_warn!("Cannot add token to cache, token cache not initialized");
    }
}

/// Removes a token from the shared token cache
///
/// # Arguments
///
/// * `api_key` - API key for the user
/// * `session_id` - Session ID for the token
pub fn remove_token_from_cache(api_key: &str, session_id: &str) {
    let cache_key = format!("{}:{}", api_key, session_id);
    let token_cache_guard = TOKEN_CACHE.lock().unwrap();
    
    // If the cache is not initialized, log a warning and return
    if let Some(token_cache) = &*token_cache_guard {
        let mut token_cache = token_cache.lock().unwrap();
        token_cache.remove(&cache_key);
        log_debug!("Removed token from cache");
    } else {
        log_warn!("Cannot remove token from cache, token cache not initialized");
    }
}

/// Gets a reference to the shared token cache if it exists
pub fn get_shared_token_cache() -> Option<SharedTokenCache> {
    TOKEN_CACHE.lock().unwrap().clone()
}

/// Initialize the JWT token cache with a shared token cache from the engine
/// This allows plugins to use the same token cache used by the auth routes
pub fn init_token_cache(shared_cache: SharedTokenCache) -> Result<(), String> {
    log_debug!("JWT Auth: About to initialize token cache");
    
    // Reset the warning flag when initializing the cache
    LOGGED_WARNING.store(false, std::sync::atomic::Ordering::SeqCst);
    
    let mut token_cache_guard = TOKEN_CACHE.lock().unwrap();
    
    // If the token cache is already set, we'll replace it - critical for proper cache sharing
    if token_cache_guard.is_some() {
        log_warn!("JWT Auth: Token cache already initialized - replacing it");
    }
    
    // Set the token cache - this is crucial for plugins to share the same token cache
    log_debug!(format!("JWT Auth: Setting token cache with address: {:p}", Arc::as_ptr(&shared_cache)).as_str());
    *token_cache_guard = Some(shared_cache);
    log_debug!("JWT Auth: Token cache initialized successfully");
    
    Ok(())
}