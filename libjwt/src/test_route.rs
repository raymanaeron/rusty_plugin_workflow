use crate::auth::JwtAuth;
use crate::models::SharedTokenCache;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Define the Foo data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foo {
    id: String,
    name: String,
    source: String,
    timestamp: u64,
}

// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

// Success response wrappers
#[derive(Debug, Serialize)]
struct FooListResponse {
    foos: Vec<Foo>,
}

#[derive(Debug, Serialize)]
struct FooResponse {
    foo: Foo,
}

#[derive(Debug, Serialize)]
struct DeleteResponse {
    message: String,
    id: String,
}

// Combined state containing both token cache and foo storage
#[derive(Clone, Debug)]
struct AppState {
    token_cache: SharedTokenCache,
    foo_storage: Arc<Mutex<HashMap<String, Foo>>>,
}

// Implement AsRef<SharedTokenCache> for AppState to enable JwtAuth extraction
impl AsRef<SharedTokenCache> for AppState {
    fn as_ref(&self) -> &SharedTokenCache {
        &self.token_cache
    }
}

// GET all foos
async fn get_all_foos(
    State(state): State<AppState>,
    _auth: JwtAuth, // JWT authentication enforced by extractor
) -> impl IntoResponse {
    // Return all foos
    let storage = state.foo_storage.lock().unwrap();
    let foos: Vec<Foo> = storage.values().cloned().collect();
    
    (StatusCode::OK, Json(FooListResponse { foos }))
}

// GET a specific foo by ID
async fn get_foo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _auth: JwtAuth, // JWT authentication enforced by extractor
) -> impl IntoResponse {
    // Check if foo exists
    let storage = state.foo_storage.lock().unwrap();
    
    match storage.get(&id) {
        Some(foo) => {
            let response = FooResponse { foo: foo.clone() };
            (StatusCode::OK, Json(response)).into_response()
        }
        None => {
            let error = ErrorResponse {
                message: format!("Foo with id {} not found", id),
            };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

// POST to create a new foo
#[derive(Debug, Deserialize)]
struct CreateFooRequest {
    name: String,
    source: String,
}

async fn create_foo(
    State(state): State<AppState>,
    _auth: JwtAuth, // JWT authentication enforced by extractor
    Json(payload): Json<CreateFooRequest>,
) -> impl IntoResponse {
    // Create new foo with unique ID
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let new_foo = Foo {
        id: id.clone(),
        name: payload.name,
        source: payload.source,
        timestamp,
    };

    // Store the foo
    let mut storage = state.foo_storage.lock().unwrap();
    storage.insert(id, new_foo.clone());

    // Return the created foo
    (StatusCode::CREATED, Json(FooResponse { foo: new_foo }))
}

// PUT to update a foo by ID
#[derive(Debug, Deserialize)]
struct UpdateFooRequest {
    name: String,
    source: String,
}

async fn update_foo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _auth: JwtAuth, // JWT authentication enforced by extractor
    Json(payload): Json<UpdateFooRequest>,
) -> impl IntoResponse {
    // Update the foo if it exists
    let mut storage = state.foo_storage.lock().unwrap();
    
    if !storage.contains_key(&id) {
        let error = ErrorResponse {
            message: format!("Foo with id {} not found", id),
        };
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let updated_foo = Foo {
        id: id.clone(),
        name: payload.name,
        source: payload.source,
        timestamp,
    };
    
    storage.insert(id, updated_foo.clone());
    
    (StatusCode::OK, Json(FooResponse { foo: updated_foo })).into_response()
}

// PATCH to partially update a foo by ID
#[derive(Debug, Deserialize)]
struct PatchFooRequest {
    name: Option<String>,
    source: Option<String>,
}

async fn patch_foo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _auth: JwtAuth, // JWT authentication enforced by extractor
    Json(payload): Json<PatchFooRequest>,
) -> impl IntoResponse {
    // Update the foo if it exists
    let mut storage = state.foo_storage.lock().unwrap();
    
    if !storage.contains_key(&id) {
        let error = ErrorResponse {
            message: format!("Foo with id {} not found", id),
        };
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }
    
    let mut current_foo = storage.get(&id).unwrap().clone();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Update only the fields that were provided
    if let Some(name) = payload.name {
        current_foo.name = name;
    }
    
    if let Some(source) = payload.source {
        current_foo.source = source;
    }
    
    current_foo.timestamp = timestamp;
    
    storage.insert(id, current_foo.clone());
    
    (StatusCode::OK, Json(FooResponse { foo: current_foo })).into_response()
}

// DELETE a foo by ID
async fn delete_foo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    _auth: JwtAuth, // JWT authentication enforced by extractor
) -> impl IntoResponse {
    // Delete the foo if it exists
    let mut storage = state.foo_storage.lock().unwrap();
    
    if !storage.contains_key(&id) {
        let error = ErrorResponse {
            message: format!("Foo with id {} not found", id),
        };
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }
    
    storage.remove(&id);
    
    let response = DeleteResponse {
        message: "Foo deleted successfully".to_string(),
        id,
    };
    (StatusCode::OK, Json(response)).into_response()
}

// Create the test router
pub fn create_test_router(token_cache: SharedTokenCache) -> Router {
    // Create in-memory storage for foo
    let foo_storage = Arc::new(Mutex::new(HashMap::<String, Foo>::new()));
    
    // Create combined app state
    let state = AppState {
        token_cache,
        foo_storage,
    };
    
    // Create the router with routes protected by JwtAuth extractor
    Router::new()
        .route("/test/foo", 
               get(get_all_foos)
               .post(create_foo))
        .route(
            "/test/foo/:id",
            get(get_foo)
            .put(update_foo)
            .patch(patch_foo)
            .delete(delete_foo),
        )
        .with_state(state)
}