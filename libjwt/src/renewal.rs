use crate::models::SharedTokenCache;
use crate::token::generate_jwt;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time;

const TOKEN_RENEWAL_INTERVAL_SECONDS: u64 = 2;  // Check every 2 seconds
const TOKEN_EXPIRY_SECONDS: u64 = 10;

/// Start the token renewal background task
///
/// This function spawns a background task that periodically checks for tokens
/// that need renewal and generates fresh tokens.
pub async fn start_renewal_task(token_cache: SharedTokenCache) {
    println!("Starting token renewal background task");
    
    let mut interval = time::interval(Duration::from_secs(TOKEN_RENEWAL_INTERVAL_SECONDS));
    
    loop {
        interval.tick().await;
        
        // Renew tokens that will expire soon
        renew_expiring_tokens(&token_cache).await;
    }
}

/// Check and renew tokens that are about to expire
async fn renew_expiring_tokens(token_cache: &SharedTokenCache) {
    let mut cache = token_cache.lock().unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let current_instant = Instant::now();
    
    // For each token in the cache, check if it needs renewal
    for (_cache_key, entry) in cache.iter_mut() {
        // Check if more than half the token lifetime has elapsed
        if current_instant.duration_since(entry.last_renewed).as_secs() > TOKEN_EXPIRY_SECONDS / 2 {
            // Generate a fresh token with the same session ID
            match generate_jwt(
                &entry.api_key, 
                &entry.session_id, 
                now, 
                now + TOKEN_EXPIRY_SECONDS
            ) {
                Ok(new_token) => {
                    // Update the token and renewal timestamp
                    entry.token = new_token;
                    entry.last_renewed = Instant::now();
                }
                Err(e) => {
                    eprintln!("Failed to renew token: {}", e);
                }
            }
        }
    }
}