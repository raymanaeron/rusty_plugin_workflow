use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (typically the API key)
    pub sub: String,
    /// Session identifier for this specific token
    pub session_id: String,
    /// Expiration time (as Unix timestamp)
    pub exp: usize,
    /// Issued at time (as Unix timestamp)
    pub iat: usize,
}

/// Request parameters for token generation
#[derive(Debug, Deserialize)]
pub struct TokenRequestParams {
    pub api_secret: String,
}

/// Response structure for token generation
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

/// Response structure for session creation
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub token: String,
}

/// Response structure for error messages
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

/// Information about an active session
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub created_at: String,
    pub last_renewed: String,
}

/// Response structure for listing sessions
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionInfo>,
}

/// Response structure for successful session revocation
#[derive(Debug, Serialize)]
pub struct RevokeResponse {
    pub message: String,
}

/// Internal entry for the token cache
#[derive(Debug, Clone)]
pub struct TokenCacheEntry {
    pub api_key: String,
    pub api_secret: String,
    pub session_id: String,
    pub token: String,
    pub created_at: Instant,
    pub last_renewed: Instant,
}

/// Type alias for the in-memory token cache
type TokenCache = HashMap<String, TokenCacheEntry>;

/// Thread-safe wrapper for session storage
#[derive(Clone, Debug)]
pub struct SharedTokenCache {
    pub memory_cache: Arc<Mutex<TokenCache>>,
    pub sqlite_storage: Option<Arc<crate::storage::SqliteSessionStorage>>,
}

impl SharedTokenCache {
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self {
            memory_cache: Arc::new(Mutex::new(HashMap::new())),
            sqlite_storage: None,
        }
    }

    /// Create a new cache with SQLite persistence
    pub async fn with_sqlite(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let sqlite = crate::storage::SqliteSessionStorage::new(db_path).await?;
        Ok(Self {
            memory_cache: Arc::new(Mutex::new(HashMap::new())),
            sqlite_storage: Some(Arc::new(sqlite)),
        })
    }
}

impl Default for SharedTokenCache {
    fn default() -> Self {
        Self::new()
    }
}