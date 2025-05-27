extern crate liblogger;
extern crate plugin_core;
extern crate liblogger_macros;

// Plugin core imports
use plugin_core::{
    log_debug, log_info, 
    declare_plugin, PluginContext, Resource, HttpMethod,
    ApiRequest, ApiResponse, error_response, cleanup_response,
    response_utils::{json_response, method_not_allowed_response},
    resource_utils::static_resource,
    jwt_utils::validate_jwt_token,
};

// Standard library
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// External dependencies
use liblogger_macros::{log_entry_exit, measure_time, initialize_logger_attributes};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tokio::runtime::Runtime;
use libws::ws_client::WsClient;
use rand;

mod network_info;

// Define our own serializable/deserializable struct for mock networks
#[derive(Serialize, Deserialize, Clone)]
struct MockNetworkInfo {
    ssid: String,
    bssid: String,
    signal: i32,
    channel: i32,
    security: String,
    frequency: f32,
}

// Define a deserializable struct for network input
#[derive(Serialize, Deserialize)]
struct NetworkInfoInput {
    ssid: String,
    bssid: Option<String>,
    signal: Option<i32>,
    channel: Option<i32>,
    security: Option<String>,
    frequency: Option<f32>,
}

// Storage for mock networks with their IDs
static MOCK_NETWORKS: Lazy<Arc<Mutex<HashMap<String, MockNetworkInfo>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// Initialize logger attributes
initialize_logger_attributes!();

// Initialize a shared runtime for all async operations in the plugin
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// Global WebSocket client instance for event-driven communication
static mut PLUGIN_WS_CLIENT: Option<Arc<Mutex<WsClient>>> = None;

// Plugin initialization hook that runs when the plugin is first loaded
#[ctor::ctor]
fn on_load() {
    // Initialize the logger for this plugin
    if let Err(e) = plugin_core::init_logger("plugin_mockwifi") {
        eprintln!("[plugin_totorial] Failed to initialize logger: {}", e);
    }
    
    log_info!("Plugin Tutorial loaded successfully");
}

// Establishes WebSocket connection for real-time event publishing/subscribing
// Automatically subscribes to the resource update event channel
pub async fn create_ws_plugin_client() {
    if let Ok(client) = WsClient::connect("plugin_mockwifi", "ws://127.0.0.1:8081/ws").await {
        let client = Arc::new(Mutex::new(client));
        
        if let Ok(mut ws_client) = client.lock() {
            ws_client.subscribe("plugin_mockwifi", "NetworkUpdated", "").await;
            log_debug!("[plugin_mockwifi] Subscribed to NetworkUpdated");
        }
        
        unsafe {
            PLUGIN_WS_CLIENT = Some(client);
        }
    }
}

// Entry point called by the plugin engine on startup
// Initializes WebSocket connection and other required resources
extern "C" fn run(_ctx: *const PluginContext) {
    println!("[plugin_mockwifi] - run");
    RUNTIME.block_on(async {
        create_ws_plugin_client().await;
    });
    
    // Initialize some mock networks for testing
    initialize_mock_networks();
}

// Initialize some mock network data
fn initialize_mock_networks() {
    let mut networks = MOCK_NETWORKS.lock().unwrap();
    
    // Add some sample networks if empty
    if networks.is_empty() {
        log_debug!("Initializing mock network data");
        
        // Add a few mock networks
        let network1 = MockNetworkInfo {
            ssid: "Home_Network".to_string(),
            bssid: "00:11:22:33:44:55".to_string(),
            signal: -65,
            channel: 6,
            security: "WPA2".to_string(),
            frequency: 2412.0,
        };
        
        let network2 = MockNetworkInfo {
            ssid: "Guest_WiFi".to_string(),
            bssid: "AA:BB:CC:DD:EE:FF".to_string(),
            signal: -72,
            channel: 11,
            security: "WPA3".to_string(),
            frequency: 2462.0,
        };
        
        let network3 = MockNetworkInfo {
            ssid: "5G_Network".to_string(),
            bssid: "11:22:33:44:55:66".to_string(),
            signal: -58,
            channel: 36,
            security: "WPA2-Enterprise".to_string(),
            frequency: 5180.0,
        };
        
        // Insert with random IDs
        networks.insert(format!("{:x}", rand::random::<u64>()), network1);
        networks.insert(format!("{:x}", rand::random::<u64>()), network2);
        networks.insert(format!("{:x}", rand::random::<u64>()), network3);
    }
}

// Convert a NetworkInfoInput to MockNetworkInfo
fn from_input_to_json(input: &NetworkInfoInput) -> MockNetworkInfo {
    MockNetworkInfo {
        ssid: input.ssid.clone(),
        bssid: input.bssid.clone().unwrap_or_else(|| format!("{:x}", rand::random::<u64>())),
        signal: input.signal.unwrap_or(-70),
        channel: input.channel.unwrap_or(1),
        security: input.security.clone().unwrap_or_else(|| "WPA2".to_string()),
        frequency: input.frequency.unwrap_or(2412.0),
    }
}

// Defines the path where static web content (HTML, CSS, JS) can be served from
// This content will be available at /mwifi/web/ in the application
extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("mwifi/web").unwrap().into_raw()
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
    let slice = static_resource("network", &METHODS);
    unsafe { *out_len = slice.len(); }
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

        // Extract ID from path if present (format: "network/{id}")
        let path_parts: Vec<&str> = path.split('/').collect();
        let (resource_path, id_opt) = if path_parts.len() >= 2 {
            (path_parts[0], Some(path_parts[1]))
        } else {
            (path, None)
        };

        match request.method {
            // GET: List all resources or get a specific one by ID
            HttpMethod::Get if resource_path == "network" => {
                log_info!("Processing network scan request");
                let networks = MOCK_NETWORKS.lock().unwrap();
                
                // If ID is provided, return that specific resource
                if let Some(id) = id_opt {
                    if let Some(item) = networks.get(id) {
                        let json = serde_json::to_string(&item).unwrap();
                        json_response(200, &json)
                    } else {
                        error_response(404, "Resource not found")
                    }
                } else {
                    // Return all mock networks
                    let json = serde_json::to_string(&*networks).unwrap();
                    log_debug!(format!("Returning all networks: {} - Context: {}", json, "plugin_mockwifi").as_str());
                    json_response(200, &json)
                }
            }
            
            // POST: Create a new resource
            HttpMethod::Post if resource_path == "network" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(network_input) = serde_json::from_slice::<NetworkInfoInput>(body) {
                    let mut networks = MOCK_NETWORKS.lock().unwrap();

                    // Convert NetworkInfoInput to NetworkInfoJson 
                    let network_json = from_input_to_json(&network_input);
                    
                    // Generate an ID for the resource
                    let resource_id = format!("{:x}", rand::random::<u64>());
                    
                    // Insert into networks with the generated ID as key
                    networks.insert(resource_id.clone(), network_json);

                    // Create response with the saved resource_id
                    let response = serde_json::json!({
                        "message": "Resource created",
                        "id": resource_id
                    });
                    log_debug!(format!("Saving a resource: {}, Context: {}", response, "plugin_mockwifi").as_str());
                    json_response(201, &serde_json::to_string(&response).unwrap())
                } else {
                    error_response(400, "Invalid data")
                }
            }
            
            // PUT: Update a resource (complete replacement)
            HttpMethod::Put if resource_path == "network" => {
                if let Some(id) = id_opt {
                    let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                    if let Ok(network_input) = serde_json::from_slice::<NetworkInfoInput>(body) {
                        let mut networks = MOCK_NETWORKS.lock().unwrap();

                        // Convert NetworkInfoInput to NetworkInfoJson
                        let network_json = from_input_to_json(&network_input);

                        if networks.contains_key(id) {
                            networks.insert(id.to_string(), network_json);
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
            HttpMethod::Delete if resource_path == "network" => {
                let mut networks = MOCK_NETWORKS.lock().unwrap();

                if let Some(id) = id_opt {
                    if networks.remove(id).is_some() {
                        json_response(200, r#"{"message": "Resource deleted"}"#)
                    } else {
                        error_response(404, "Resource not found")
                    }
                } else {
                    // Clear all resources
                    networks.clear();
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
    "plugin_mockwifi",
    "mwifi",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);
