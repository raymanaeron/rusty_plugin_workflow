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
    let session_id = Uuid::new_v4().to_string();
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    match generate_jwt(&api_key, &session_id, now, now + TOKEN_EXPIRY_SECONDS) {
        Ok(token) => {
            let cache_key = format!("{}:{}", api_key, session_id);
            let entry = TokenCacheEntry {
                api_key: api_key.clone(),
                api_secret: params.api_secret.clone(),
                session_id: session_id.clone(),
                token: token.clone(),
                created_at: Instant::now(),
                last_renewed: Instant::now(),
            };

            // Store in memory cache
            token_cache.memory_cache.lock().await.insert(cache_key.clone(), entry.clone());

            // If SQLite storage is enabled, store there as well
            if let Some(sqlite) = &token_cache.sqlite_storage {
                if let Err(e) = sqlite.create_session(cache_key, entry).await {
                    eprintln!("Error storing session in SQLite: {}", e);
                }
            }
            
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
    let cache_key = format!("{}:{}", api_key, session_id);
    let found_session = token_cache.memory_cache.lock().await.get(&cache_key).cloned();

    let found_session = if found_session.is_none() {
        if let Some(sqlite) = &token_cache.sqlite_storage {
            match sqlite.get_session(&cache_key).await {
                Ok(Some(entry)) => {
                    token_cache.memory_cache.lock().await.insert(cache_key.clone(), entry.clone());
                    Some(entry)
                }
                _ => None,
            }
        } else {
            None
        }
    } else {
        found_session
    };

    match found_session {
        Some(_entry) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            match generate_jwt(&api_key, &session_id, now, now + TOKEN_EXPIRY_SECONDS) {
                Ok(token) => {
                    if let Some(sqlite) = &token_cache.sqlite_storage {
                        if let Err(e) = sqlite.update_session_token(&cache_key, &token).await {
                            eprintln!("Error updating token in SQLite: {}", e);
                        }
                    }
                    (StatusCode::OK, Json(TokenResponse { token })).into_response()
                }
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
    let mut sessions = Vec::new();
    let now = Instant::now();

    {
        let cache = token_cache.memory_cache.lock().await;
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
    }

    if let Some(sqlite) = &token_cache.sqlite_storage {
        match sqlite.list_sessions_by_api_key(&api_key).await {
            Ok(sqlite_sessions) => {
                for entry in sqlite_sessions {
                    if !sessions.iter().any(|s| s.session_id == entry.session_id) {
                        let created_duration = now.duration_since(entry.created_at);
                        let renewed_duration = now.duration_since(entry.last_renewed);
                        
                        sessions.push(SessionInfo {
                            session_id: entry.session_id,
                            created_at: format!("{:?} ago", created_duration),
                            last_renewed: format!("{:?} ago", renewed_duration),
                        });
                    }
                }
            }
            Err(e) => eprintln!("Error retrieving sessions from SQLite: {}", e),
        }
    }
    
    (StatusCode::OK, Json(SessionListResponse { sessions }))
}

/// Revoke/delete a specific session
pub async fn revoke_session(
    State(token_cache): State<SharedTokenCache>,
    Path((api_key, session_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let cache_key = format!("{}:{}", api_key, session_id);
    let mut deleted = false;

    {
        let mut cache = token_cache.memory_cache.lock().await;
        if cache.remove(&cache_key).is_some() {
            deleted = true;
        }
    }

    if let Some(sqlite) = &token_cache.sqlite_storage {
        match sqlite.delete_session(&cache_key).await {
            Ok(true) => deleted = true,
            Ok(false) => {}
            Err(e) => eprintln!("Error deleting session from SQLite: {}", e),
        }
    }

    if deleted {
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
pub async fn validate_session_token(
    token_cache: &SharedTokenCache,
    token: &str,
) -> Result<(String, String), String> {
    match validate_jwt(token) {
        Ok(claims) => {
            let cache_key = format!("{}:{}", claims.sub, claims.session_id);
            let found_in_memory = token_cache.memory_cache.lock().await.contains_key(&cache_key);
            
            if found_in_memory {
                return Ok((claims.sub, claims.session_id));
            }
            
            Err("Invalid session".to_string())
        }
        Err(e) => Err(format!("Invalid token: {}", e)),
    }
}