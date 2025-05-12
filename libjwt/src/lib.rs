// Export modules
pub mod models;
pub mod token;
pub mod session;
pub mod renewal;
pub mod routes;
pub mod auth;
pub mod test_route;
pub mod storage;
pub mod jwt_manager;

// Re-export important items for easier usage
pub use token::{generate_jwt, validate_jwt, get_api_key_from_token};
pub use models::Claims;
pub use models::SharedTokenCache;
pub use routes::{create_auth_router, create_auth_router_with_cache};
pub use test_route::create_test_router;
pub use auth::{JwtAuth, AuthClaims, AuthErrorResponse};
pub use renewal::start_renewal_task;
pub use jwt_manager::JwtManager;