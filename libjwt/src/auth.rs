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
    // Extract the Authorization header
    let auth_header = match headers.get("Authorization") {
        Some(value) => match value.to_str() {
            Ok(v) => v,
            Err(_) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        message: "Invalid Authorization header format".to_string(),
                    }),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    message: "Missing Authorization header".to_string(),
                }),
            ))
        }
    };

    // Check if it starts with "Bearer " and extract the token
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
    match validate_jwt(token) {
        Ok(claims) => {
            let cache = token_cache.lock().unwrap();
            let cache_key = format!("{}:{}", claims.sub, claims.session_id);

            if cache.contains_key(&cache_key) {
                Ok((claims.sub, claims.session_id))
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        message: "Invalid session".to_string(),
                    }),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                message: format!("Invalid JWT token: {}", e),
            }),
        )),
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
        // Get token cache from state
        let token_cache = state.as_ref();

        // Extract authorization header
        let headers = &parts.headers;

        // Validate token using our helper function
        match extract_and_validate_token(token_cache, headers).await {
            Ok((api_key, session_id)) => {
                // Return successful authentication
                Ok(JwtAuth(AuthClaims {
                    api_key,
                    session_id,
                }))
            }
            Err((status, json_error)) => {
                // Return error response
                Err((status, json_error).into_response())
            }
        }
    }
}