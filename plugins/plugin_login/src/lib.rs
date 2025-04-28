extern crate plugin_core;

use plugin_core::{
    ApiRequest, ApiResponse, HttpMethod, PluginContext, Resource,
    declare_plugin,
    error_response,
    response_utils::{json_response, method_not_allowed_response},
    resource_utils::static_resource,
    cleanup_response,
};
use std::sync::{Arc, Mutex};
use ws_server::ws_client::WsClient;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;
use std::os::raw::c_char;
use std::ffi::{CString, CStr};
use std::ptr;
use serde::{Serialize, Deserialize};

// Shared Runtime for async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// Shared WebSocket client
static mut PLUGIN_WS_CLIENT: Option<Arc<Mutex<WsClient>>> = None;

// Define your data structure - using CamelCase for type name
#[derive(Serialize, Deserialize, Clone, Default)]
struct User {
    id: String,
    field1: String,
    field2: bool,
}

// Shared state - using a HashMap to store multiple items by ID
static STATE: Lazy<Mutex<std::collections::HashMap<String, User>>> = Lazy::new(|| {
    Mutex::new(std::collections::HashMap::new())
});

#[ctor::ctor]
fn on_load() {
    println!("[plugin_login] >>> LOADED");
}

pub async fn create_ws_plugin_client() {
    if let Ok(client) = WsClient::connect("plugin_login", "ws://127.0.0.1:8081/ws").await {
        let client = Arc::new(Mutex::new(client));
        
        if let Ok(mut ws_client) = client.lock() {
            ws_client.subscribe("plugin_login", "UserUpdated", "").await;
            println!("[plugin_login] Subscribed to UserUpdated");
        }
        
        unsafe {
            PLUGIN_WS_CLIENT = Some(client);
        }
    }
}

extern "C" fn run(_ctx: *const PluginContext) {
    println!("[plugin_login] - run");
    RUNTIME.block_on(async {
        create_ws_plugin_client().await;
    });
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("login/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 4] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
    ];
    let slice = static_resource("user", &METHODS);
    unsafe { *out_len = slice.len(); }
    slice.as_ptr()
}

extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let request = &*req;
        let path = if request.path.is_null() {
            "<null>"
        } else {
            CStr::from_ptr(request.path).to_str().unwrap_or("<invalid>")
        };

        // Extract ID from path if present (format: "user/ID")
        let path_parts: Vec<&str> = path.split('/').collect();
        let (resource_path, id_opt) = if path_parts.len() >= 2 {
            (path_parts[0], Some(path_parts[1]))
        } else {
            (path, None)
        };

        match request.method {
            // GET: List all resources or get a specific one by ID
            HttpMethod::Get if resource_path == "user" => {
                let state = STATE.lock().unwrap();
                
                // If ID is provided, return that specific resource
                if let Some(id) = id_opt {
                    if let Some(item) = state.get(id) {
                        let json = serde_json::to_string(&item).unwrap();
                        json_response(200, &json)
                    } else {
                        error_response(404, "Resource not found")
                    }
                } else {
                    // Return all resources
                    let json = serde_json::to_string(&*state).unwrap();
                    json_response(200, &json)
                }
            }

            // POST: Create a new resource
            HttpMethod::Post if resource_path == "user" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(mut data) = serde_json::from_slice::<User>(body) {
                    let mut state = STATE.lock().unwrap();
                    
                    // If ID is empty, generate one
                    if data.id.is_empty() {
                        data.id = format!("{:x}", rand::random::<u64>());
                    }
                    
                    // Clone the ID for use in the response
                    let resource_id = data.id.clone();
                    
                    // Insert into state
                    state.insert(data.id.clone(), data.clone());
                    
                    // Get WebSocket client safely without creating a shared reference to static
                    let client_clone = {
                        // Get a raw pointer to the static
                        let ws_client_ptr = ptr::addr_of!(PLUGIN_WS_CLIENT);
                        // Dereference the raw pointer to get Option<Arc<Mutex<WsClient>>>
                        (*ws_client_ptr).as_ref().map(Arc::clone)
                    };
                    
                    if let Some(client) = client_clone {
                        let timestamp = chrono::Utc::now().to_rfc3339();
                        let data_clone = data.clone(); // Clone data before moving
                        
                        // Use spawn_blocking to handle the non-Send MutexGuard
                        RUNTIME.spawn(async move {
                            let timestamp_clone = timestamp.clone();
                            let payload = serde_json::to_string(&data_clone).unwrap();
                            
                            tokio::task::spawn_blocking(move || {
                                if let Ok(mut ws_client) = client.lock() {
                                    let rt = tokio::runtime::Handle::current();
                                    let _ = rt.block_on(ws_client.publish(
                                        "plugin_login", 
                                        "UserUpdated", 
                                        &payload,
                                        &timestamp_clone
                                    ));
                                }
                            }).await.ok();
                        });
                    }
                    
                    // Create response with the saved resource_id
                    let response = serde_json::json!({
                        "message": "Resource created",
                        "id": resource_id
                    });
                    json_response(201, &serde_json::to_string(&response).unwrap())
                } else {
                    error_response(400, "Invalid data")
                }
            }

            // PUT: Update a resource (complete replacement)
            HttpMethod::Put if resource_path == "user" => {
                if let Some(id) = id_opt {
                    let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                    if let Ok(mut data) = serde_json::from_slice::<User>(body) {
                        let mut state = STATE.lock().unwrap();
                        
                        // Ensure the ID in the URL matches the resource
                        data.id = id.to_string();
                        
                        if state.contains_key(id) {
                            state.insert(id.to_string(), data.clone());
                            
                            // Get WebSocket client safely without creating a shared reference to static
                            let client_clone = {
                                // Get a raw pointer to the static
                                let ws_client_ptr = ptr::addr_of!(PLUGIN_WS_CLIENT);
                                // Dereference the raw pointer to get Option<Arc<Mutex<WsClient>>>
                                (*ws_client_ptr).as_ref().map(Arc::clone)
                            };
                            
                            if let Some(client) = client_clone {
                                let timestamp = chrono::Utc::now().to_rfc3339();
                                let data_clone = data.clone();
                                
                                RUNTIME.spawn(async move {
                                    let timestamp_clone = timestamp.clone();
                                    let payload = serde_json::to_string(&data_clone).unwrap();
                                    
                                    tokio::task::spawn_blocking(move || {
                                        if let Ok(mut ws_client) = client.lock() {
                                            let rt = tokio::runtime::Handle::current();
                                            let _ = rt.block_on(ws_client.publish(
                                                "plugin_login", 
                                                "UserUpdated", 
                                                &payload,
                                                &timestamp_clone
                                            ));
                                        }
                                    }).await.ok();
                                });
                            }
                            
                            json_response(200, r#"{"message": "Resource updated"}"#)
                        } else {
                            error_response(404, "Resource not found")
                        }
                    } else {
                        error_response(400, "Invalid data")
                    }
                } else {
                    error_response(400, "Resource ID required")
                }
            }

            // DELETE: Remove a resource
            HttpMethod::Delete if resource_path == "user" => {
                let mut state = STATE.lock().unwrap();
                
                if let Some(id) = id_opt {
                    let id_string = id.to_string(); // Clone ID for use in async block
                    
                    if state.remove(id).is_some() {
                        // Get WebSocket client safely without creating a shared reference to static
                        let client_clone = {
                            // Get a raw pointer to the static
                            let ws_client_ptr = ptr::addr_of!(PLUGIN_WS_CLIENT);
                            // Dereference the raw pointer to get Option<Arc<Mutex<WsClient>>>
                            (*ws_client_ptr).as_ref().map(Arc::clone)
                        };
                        
                        if let Some(client) = client_clone {
                            let timestamp = chrono::Utc::now().to_rfc3339();
                            
                            RUNTIME.spawn(async move {
                                let timestamp_clone = timestamp.clone();
                                let event_data = serde_json::json!({ "id": id_string, "deleted": true });
                                let payload = serde_json::to_string(&event_data).unwrap();
                                
                                tokio::task::spawn_blocking(move || {
                                    if let Ok(mut ws_client) = client.lock() {
                                        let rt = tokio::runtime::Handle::current();
                                        let _ = rt.block_on(ws_client.publish(
                                            "plugin_login", 
                                            "UserUpdated", 
                                            &payload,
                                            &timestamp_clone
                                        ));
                                    }
                                }).await.ok();
                            });
                        }
                        
                        json_response(200, r#"{"message": "Resource deleted"}"#)
                    } else {
                        error_response(404, "Resource not found")
                    }
                } else {
                    // Clear all resources
                    state.clear();
                    json_response(200, r#"{"message": "All resources deleted"}"#)
                }
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

declare_plugin!(
    "plugin_login",
    "login",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);