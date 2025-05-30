extern crate liblogger;
extern crate plugin_core;
extern crate liblogger_macros;

// Plugin core imports
use plugin_core::{
    log_debug,
    log_info,
    declare_plugin,
    PluginContext,
    Resource,
    HttpMethod,
    ApiRequest,
    ApiResponse,
    error_response,
    cleanup_response,
    response_utils::{ json_response, method_not_allowed_response },
    resource_utils::static_resource,
};
use plugin_core::jwt_utils::validate_jwt_token;

// Standard library
use std::ffi::{ CString, CStr };
use std::os::raw::c_char;
use std::ptr;
use std::sync::{ Arc, Mutex };

// External dependencies
use liblogger_macros::{ log_entry_exit, measure_time, initialize_logger_attributes };
use once_cell::sync::Lazy;
use serde::{ Serialize, Deserialize };
use tokio::runtime::Runtime;
use libws::ws_client::WsClient;

// Initialize logger attributes
initialize_logger_attributes!();

// Initialize a shared runtime for all async operations in the plugin
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// Global WebSocket client instance for event-driven communication
static mut PLUGIN_WS_CLIENT: Option<Arc<Mutex<WsClient>>> = None;

// Resource data model definition
#[derive(Serialize, Deserialize, Clone, Default)]
struct Summary {
    id: String,
    field1: String,
    field2: bool,
}

// In-memory database implemented as a thread-safe HashMap
// Keys are resource IDs, values are resource instances
static STATE: Lazy<Mutex<std::collections::HashMap<String, Summary>>> = Lazy::new(|| {
    Mutex::new(std::collections::HashMap::new())
});

// Plugin initialization hook that runs when the plugin is first loaded
#[ctor::ctor]
fn on_load() {
    // Initialize the logger for this plugin
    if let Err(e) = plugin_core::init_logger("plugin_finish") {
        eprintln!("[plugin_totorial] Failed to initialize logger: {}", e);
    }

    log_info!("Plugin Tutorial loaded successfully");
}

// Establishes WebSocket connection for real-time event publishing/subscribing
// Automatically subscribes to the resource update event channel
pub async fn create_ws_plugin_client() {
    if let Ok(client) = WsClient::connect("plugin_finish", "ws://127.0.0.1:8081/ws").await {
        let client = Arc::new(Mutex::new(client));

        if let Ok(mut ws_client) = client.lock() {
            ws_client.subscribe("plugin_finish", "SummaryUpdated", "").await;
            log_debug!("[plugin_finish] Subscribed to SummaryUpdated");
        }

        unsafe {
            PLUGIN_WS_CLIENT = Some(client);
        }
    }
}

// Entry point called by the plugin engine on startup
// Initializes WebSocket connection and other required resources
extern "C" fn run(_ctx: *const PluginContext) {
    println!("[plugin_finish] - run");
    RUNTIME.block_on(async {
        create_ws_plugin_client().await;
    });
}

// Defines the path where static web content (HTML, CSS, JS) can be served from
// This content will be available at /finish/web/ in the application
extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("finish/web").unwrap().into_raw()
}

// Registers API endpoints that this plugin will handle
// Defines which HTTP methods are accepted for the resource
extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 4] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
    ];
    let slice = static_resource("summary", &METHODS);
    unsafe {
        *out_len = slice.len();
    }
    slice.as_ptr()
}

// Main request handler implementing RESTful API operations
// Processes GET, POST, PUT, and DELETE requests for the resource
#[log_entry_exit]
#[measure_time]
extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let request = &*req;

        // Validate JWT token using the shared utility function
        if let Err(response) = validate_jwt_token(request) {
            return response;
        }

        let path = if request.path.is_null() {
            "<null>"
        } else {
            CStr::from_ptr(request.path).to_str().unwrap_or("<invalid>")
        };

        // Extract ID from path if present (format: "summary/{id}")
        let path_parts: Vec<&str> = path.split('/').collect();
        let (resource_path, id_opt) = if path_parts.len() >= 2 {
            (path_parts[0], Some(path_parts[1]))
        } else {
            (path, None)
        };

        match request.method {
            // GET: List all resources or get a specific one by ID
            HttpMethod::Get if resource_path == "summary" => {
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
                    log_debug!(
                        format!(
                            "Returning all resources: {} - Context: {}",
                            json,
                            "plugin_finish"
                        ).as_str()
                    );
                    json_response(200, &json)
                }
            }

            // POST: Create a new resource
            HttpMethod::Post if resource_path == "summary" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(mut data) = serde_json::from_slice::<Summary>(body) {
                    let mut state = STATE.lock().unwrap();

                    // If ID is empty, generate one
                    if data.id.is_empty() {
                        data.id = format!("{:x}", rand::random::<u64>());
                    }

                    // Clone the ID for use in the response
                    let resource_id = data.id.clone();

                    // Insert into state
                    state.insert(data.id.clone(), data.clone());

                    // Create response with the saved resource_id
                    let response =
                        serde_json::json!({
                        "message": "Resource created",
                        "id": resource_id
                    });
                    log_debug!(
                        format!(
                            "Saving a resource: {}, Context: {}",
                            response,
                            "plugin_finish"
                        ).as_str()
                    );
                    json_response(201, &serde_json::to_string(&response).unwrap())
                } else {
                    error_response(400, "Invalid data")
                }
            }

            // PUT: Update a resource (complete replacement)
            HttpMethod::Put if resource_path == "summary" => {
                if let Some(id) = id_opt {
                    let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                    if let Ok(mut data) = serde_json::from_slice::<Summary>(body) {
                        let mut state = STATE.lock().unwrap();

                        // Ensure the ID in the URL matches the resource
                        data.id = id.to_string();

                        if state.contains_key(id) {
                            state.insert(id.to_string(), data.clone());
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
            HttpMethod::Delete if resource_path == "summary" => {
                let mut state = STATE.lock().unwrap();

                if let Some(id) = id_opt {
                    if state.remove(id).is_some() {
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

// Memory cleanup function called by the plugin engine
// Prevents memory leaks when responses are no longer needed
extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

// Plugin declaration macro that registers this module with the plugin system
// Defines name, route, and callback functions used by the plugin loader
declare_plugin!(
    "plugin_finish",
    "finish",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);
