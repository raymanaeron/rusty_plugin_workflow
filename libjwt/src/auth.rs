use crate::models::SharedTokenCache;
use crate::token::validate_jwt;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

// Error response structure for auth errors
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub message: String,
}

// Auth claims from JWT token
#[derive(Debug, Clone)]
pub struct AuthClaims {
    pub api_key: String,
    pub session_id: String,
}

/// JWT authentication extractor
#[derive(Debug, Clone)]
pub struct JwtAuth(pub AuthClaims);

/// Extract and validate JWT token from request headers
pub async fn extract_and_validate_token(
    token_cache: &SharedTokenCache,
    headers: &HeaderMap,
) -> Result<(String, String), (StatusCode, Json<AuthErrorResponse>)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    message: "Missing Authorization header".to_string(),
                }),
            )
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                message: "Authorization header must use Bearer scheme".to_string(),
            }),
        ));
    }

    let token = &auth_header[7..]; // Skip "Bearer "

    // Validate the JWT token
    let claims = validate_jwt(token).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                message: format!("Invalid token: {}", e),
            }),
        )
    })?;

    let cache_key = format!("{}:{}", claims.sub, claims.session_id);

    // Try memory cache first
    {
        let cache = token_cache.memory_cache.lock().await;
        if cache.contains_key(&cache_key) {
            return Ok((claims.sub, claims.session_id));
        }
    }

    // If not in memory and SQLite is enabled, try SQLite
    if let Some(sqlite) = &token_cache.sqlite_storage {
        match sqlite.get_session(&cache_key).await {
            Ok(Some(entry)) => {
                // Cache the entry in memory for future use
                token_cache.memory_cache.lock().await.insert(cache_key, entry);
                Ok((claims.sub, claims.session_id))
            }
            Ok(None) => Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    message: "Invalid session".to_string(),
                }),
            )),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    message: format!("Storage error: {}", e),
                }),
            )),
        }
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                message: "Invalid session".to_string(),
            }),
        ))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
    S: AsRef<SharedTokenCache>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token_cache = state.as_ref();
        let headers = &parts.headers;

        match extract_and_validate_token(token_cache, headers).await {
            Ok((api_key, session_id)) => Ok(JwtAuth(AuthClaims { api_key, session_id })),
            Err(response) => Err(response.into_response()),
        }
    }
}