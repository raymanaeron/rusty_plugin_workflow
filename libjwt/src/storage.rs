use std::time::{SystemTime, UNIX_EPOCH};
use tokio_rusqlite::Connection;
use crate::models::TokenCacheEntry;
use std::time::Instant;

#[derive(Debug)]
pub struct StorageError(pub String);

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StorageError {}

#[derive(Debug)]
pub struct SqliteSessionStorage {
    conn: Connection,
}

impl SqliteSessionStorage {
    pub async fn new(db_path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(db_path).await
            .map_err(|e| StorageError(e.to_string()))?;
        
        // Create sessions table if it doesn't exist
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS sessions (
                    cache_key TEXT PRIMARY KEY,
                    api_key TEXT NOT NULL,
                    api_secret TEXT NOT NULL,
                    session_id TEXT NOT NULL,
                    token TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    last_renewed INTEGER NOT NULL
                )",
                [],
            )
        }).await.map_err(|e| StorageError(e.to_string()))?;

        Ok(Self { conn })
    }

    pub async fn create_session(&self, cache_key: String, entry: TokenCacheEntry) -> Result<(), StorageError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO sessions 
                (cache_key, api_key, api_secret, session_id, token, created_at, last_renewed) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                [
                    &cache_key,
                    &entry.api_key,
                    &entry.api_secret,
                    &entry.session_id,
                    &entry.token,
                    &now.to_string(),
                    &now.to_string(),
                ],
            )
        }).await.map_err(|e| StorageError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_session(&self, cache_key: &str) -> Result<Option<TokenCacheEntry>, StorageError> {
        let cache_key = cache_key.to_string();
        let result = self.conn.call(move |conn| {
            let result = conn.query_row(
                "SELECT api_key, api_secret, session_id, token, created_at, last_renewed 
                FROM sessions WHERE cache_key = ?1",
                [&cache_key],
                |row| {
                    let created_at_secs: i64 = row.get(4)?;
                    let last_renewed_secs: i64 = row.get(5)?;
                    
                    Ok(TokenCacheEntry {
                        api_key: row.get(0)?,
                        api_secret: row.get(1)?,
                        session_id: row.get(2)?,
                        token: row.get(3)?,
                        created_at: Instant::now() - std::time::Duration::from_secs(
                            (SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64 - created_at_secs) as u64
                        ),
                        last_renewed: Instant::now() - std::time::Duration::from_secs(
                            (SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64 - last_renewed_secs) as u64
                        ),
                    })
                },
            );
            
            match result {
                Ok(entry) => Ok(Some(entry)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await.map_err(|e| StorageError(e.to_string()))?;

        Ok(result)
    }

    pub async fn list_sessions_by_api_key(&self, api_key: &str) -> Result<Vec<TokenCacheEntry>, StorageError> {
        let api_key = api_key.to_string();
        let entries = self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT api_key, api_secret, session_id, token, created_at, last_renewed 
                FROM sessions WHERE api_key = ?1"
            )?;
            
            let rows = stmt.query_map([&api_key], |row| {
                let created_at_secs: i64 = row.get(4)?;
                let last_renewed_secs: i64 = row.get(5)?;
                
                Ok(TokenCacheEntry {
                    api_key: row.get(0)?,
                    api_secret: row.get(1)?,
                    session_id: row.get(2)?,
                    token: row.get(3)?,
                    created_at: Instant::now() - std::time::Duration::from_secs(
                        (SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64 - created_at_secs) as u64
                    ),
                    last_renewed: Instant::now() - std::time::Duration::from_secs(
                        (SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64 - last_renewed_secs) as u64
                    ),
                })
            })?;

            rows.collect::<Result<Vec<_>, _>>()
        }).await.map_err(|e| StorageError(e.to_string()))?;

        Ok(entries)
    }

    pub async fn delete_session(&self, cache_key: &str) -> Result<bool, StorageError> {
        let cache_key = cache_key.to_string();
        let rows_affected = self.conn.call(move |conn| {
            conn.execute("DELETE FROM sessions WHERE cache_key = ?1", [&cache_key])
        }).await.map_err(|e| StorageError(e.to_string()))?;

        Ok(rows_affected > 0)
    }

    pub async fn update_session_token(&self, cache_key: &str, new_token: &str) -> Result<(), StorageError> {
        let (cache_key, new_token) = (cache_key.to_string(), new_token.to_string());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.call(move |conn| {
            conn.execute(
                "UPDATE sessions SET token = ?1, last_renewed = ?2 WHERE cache_key = ?3",
                [&new_token, &now.to_string(), &cache_key],
            )
        }).await.map_err(|e| StorageError(e.to_string()))?;

        Ok(())
    }
}