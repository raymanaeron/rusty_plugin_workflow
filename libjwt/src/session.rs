use crate::models::{
    ErrorResponse, RevokeResponse, SessionInfo, SessionListResponse, SessionResponse, 
    SharedTokenCache, TokenCacheEntry, TokenRequestParams, TokenResponse
};
use crate::token::{generate_jwt, validate_jwt};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const TOKEN_EXPIRY_SECONDS: u64 = 10;

/// Create a new session for an API key
pub async fn create_session(
    State(token_cache): State<SharedTokenCache>,
    Path(api_key): Path<String>,
    Json(params): Json<TokenRequestParams>,
) -> impl IntoResponse {
    // For this demo app, we accept any API key and secret
    // In a real application, you would validate these against a database
    
    let session_id = Uuid::new_v4().to_string();
    
    // Generate JWT token with session ID included
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    match generate_jwt(&api_key, &session_id, now, now + TOKEN_EXPIRY_SECONDS) {
        Ok(token) => {
            // Store in token cache with TTL
            let mut cache = token_cache.lock().unwrap();
            
            // Cache key is a combination of API key and session ID
            let cache_key = format!("{}:{}", api_key, session_id);
            
            cache.insert(
                cache_key,
                TokenCacheEntry {
                    api_key: api_key.clone(),
                    api_secret: params.api_secret.clone(),
                    session_id: session_id.clone(),
                    token: token.clone(),
                    created_at: Instant::now(),
                    last_renewed: Instant::now(),
                },
            );
            
            (
                StatusCode::CREATED,
                Json(SessionResponse {
                    session_id,
                    token,
                }),
            ).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: format!("Error generating token: {}", e),
            }),
        ).into_response()
    }
}

/// Get a token for a specific session
pub async fn get_session_token(
    State(token_cache): State<SharedTokenCache>,
    Path((api_key, session_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let cache = token_cache.lock().unwrap();
    let cache_key = format!("{}:{}", api_key, session_id);
    
    match cache.get(&cache_key) {
        Some(_entry) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            // Generate a fresh token
            match generate_jwt(&api_key, &session_id, now, now + TOKEN_EXPIRY_SECONDS) {
                Ok(token) => (StatusCode::OK, Json(TokenResponse { token })).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        message: format!("Error generating token: {}", e),
                    }),
                ).into_response()
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                message: format!("No active session found for the provided session ID"),
            }),
        ).into_response()
    }
}

/// List all active sessions for an API key
pub async fn list_sessions(
    State(token_cache): State<SharedTokenCache>,
    Path(api_key): Path<String>,
) -> impl IntoResponse {
    let cache = token_cache.lock().unwrap();
    
    let mut sessions = Vec::new();
    let now = Instant::now();
    
    // Filter sessions by API key and collect session info
    for (_, entry) in cache.iter() {
        if entry.api_key == api_key {
            let created_duration = now.duration_since(entry.created_at);
            let renewed_duration = now.duration_since(entry.last_renewed);
            
            sessions.push(SessionInfo {
                session_id: entry.session_id.clone(),
                created_at: format!("{:?} ago", created_duration),
                last_renewed: format!("{:?} ago", renewed_duration),
            });
        }
    }
    
    (StatusCode::OK, Json(SessionListResponse { sessions }))
}

/// Revoke/delete a specific session
pub async fn revoke_session(
    State(token_cache): State<SharedTokenCache>,
    Path((api_key, session_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut cache = token_cache.lock().unwrap();
    let cache_key = format!("{}:{}", api_key, session_id);
    
    if cache.remove(&cache_key).is_some() {
        (
            StatusCode::OK,
            Json(RevokeResponse {
                message: "Session successfully revoked".to_string(),
            }),
        ).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                message: "Session not found".to_string(),
            }),
        ).into_response()
    }
}

/// Check if a token is valid and associated with an active session
pub fn validate_session_token(
    token_cache: &SharedTokenCache,
    token: &str,
) -> Result<(String, String), String> {
    match validate_jwt(token) {
        Ok(claims) => {
            let cache = token_cache.lock().unwrap();
            let cache_key = format!("{}:{}", claims.sub, claims.session_id);
            
            if cache.contains_key(&cache_key) {
                Ok((claims.sub, claims.session_id))
            } else {
                Err("Invalid session".to_string())
            }
        }
        Err(e) => Err(format!("Invalid token: {}", e)),
    }
}