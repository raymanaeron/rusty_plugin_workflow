use crate::models::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::error::Error;

#[derive(Debug)]
pub struct TokenError(String);

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TokenError {}

impl From<Box<dyn Error>> for TokenError {
    fn from(error: Box<dyn Error>) -> Self {
        TokenError(error.to_string())
    }
}

/// Generates a JWT token with the specified parameters
///
/// # Arguments
///
/// * `api_key` - The API key to use as subject in the JWT
/// * `session_id` - The session ID to include in the claims
/// * `iat` - Issued at timestamp
/// * `exp` - Expiration timestamp
///
/// # Returns
///
/// A Result containing the JWT token string or an error
pub fn generate_jwt(api_key: &str, session_id: &str, iat: u64, exp: u64) -> Result<String, TokenError> {
    // Check if the supplied api_key is not empty
    if api_key.is_empty() || session_id.is_empty() {
        return Err(TokenError("API key and session ID cannot be empty".to_string()));
    }
    
    // Create claims
    let claims = Claims {
        sub: api_key.to_string(),
        session_id: session_id.to_string(),
        exp: exp as usize,
        iat: iat as usize,
    };
    
    // Generate the JWT using a hardcoded secret for demonstration
    // In a real application, this would come from a secure configuration
    let secret = "jwt_secret_do_not_use_in_production";
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).map_err(|e| TokenError(e.to_string()))
}

/// Validates a JWT token and returns the claims
///
/// # Arguments
///
/// * `token` - The JWT token to validate
///
/// # Returns
///
/// A Result containing the Claims if the token is valid
pub fn validate_jwt(token: &str) -> Result<Claims, TokenError> {
    let secret = "jwt_secret_do_not_use_in_production";
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    )
    .map(|token_data| token_data.claims)
    .map_err(|e| TokenError(e.to_string()))
}

/// Extracts the API key (subject) from a JWT token
///
/// # Arguments
///
/// * `token` - The JWT token
///
/// # Returns
///
/// A Result containing the API key or an error
pub fn get_api_key_from_token(token: &str) -> Result<String, TokenError> {
    let claims = validate_jwt(token)?;
    Ok(claims.sub)
}