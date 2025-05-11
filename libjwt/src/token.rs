use crate::models::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::error::Error;
use std::fmt;

/// Error type for JWT operations
#[derive(Debug)]
pub struct JwtError {
    pub message: String,
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JWT error: {}", self.message)
    }
}

impl Error for JwtError {}

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
pub fn generate_jwt(api_key: &str, session_id: &str, iat: u64, exp: u64) -> Result<String, Box<dyn Error>> {
    // Check if the supplied api_key is not empty
    if api_key.is_empty() || session_id.is_empty() {
        return Err(Box::new(JwtError {
            message: "API key and session ID cannot be empty".to_string(),
        }));
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
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )?;
    
    Ok(token)
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
pub fn validate_jwt(token: &str) -> Result<Claims, Box<dyn Error>> {
    let validation = Validation::default();
    // Using the same hardcoded secret as in generate_jwt
    let secret = "jwt_secret_do_not_use_in_production";
    let key = DecodingKey::from_secret(secret.as_bytes());
    
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => Err(Box::new(JwtError {
            message: format!("Token validation failed: {}", e),
        })),
    }
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
pub fn get_api_key_from_token(token: &str) -> Result<String, Box<dyn Error>> {
    let claims = validate_jwt(token)?;
    Ok(claims.sub)
}