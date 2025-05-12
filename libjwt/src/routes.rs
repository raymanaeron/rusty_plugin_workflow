use crate::models::SharedTokenCache;
use crate::session::{create_session, get_session_token, list_sessions, revoke_session};
use axum::{
    routing::{get, post},
    Router,
};

/// Create an authentication router with all JWT session management endpoints
///
/// This function returns a configured Router with the following endpoints:
/// - POST /auth/{api_key}/sessions - Create a new session
/// - GET /auth/{api_key}/sessions - List all sessions
/// - GET /auth/{api_key}/sessions/{session_id} - Get token for a specific session
/// - DELETE /auth/{api_key}/sessions/{session_id} - Revoke a session
pub fn create_auth_router() -> Router {
    // Create a shared token cache
    let token_cache: SharedTokenCache = SharedTokenCache::new();
    create_auth_router_with_cache(token_cache)
}

/// Create an authentication router with the provided shared token cache and optional SQLite storage
pub async fn create_auth_router_with_sqlite(
    db_path: &str,
) -> Result<Router, Box<dyn std::error::Error>> {
    // Create a shared token cache with SQLite storage
    let token_cache = SharedTokenCache::with_sqlite(db_path).await?;
    Ok(create_auth_router_with_cache(token_cache))
}

/// Create an authentication router with the provided shared token cache
///
/// This is useful when you need to share the token cache with other components,
/// like a token renewal task.
pub fn create_auth_router_with_cache(token_cache: SharedTokenCache) -> Router {
    Router::new()
        .route(
            "/auth/:api_key/sessions",
            post(create_session).get(list_sessions),
        )
        .route(
            "/auth/:api_key/sessions/:session_id",
            get(get_session_token).delete(revoke_session),
        )
        .with_state(token_cache)
}