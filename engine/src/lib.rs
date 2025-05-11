//! # Engine Core Library
//!
//! This crate is the main entry point for the plugin-based OOBE (Out-Of-Box Experience) engine.
//! It provides the runtime and infrastructure for loading plugins, managing routes, handling WebSocket
//! and REST API communication, and serving static files for the web UI.
//!
//! ## Features
//! - **Plugin loading and lifecycle management:** Dynamically loads plugins and manages their state.
//! - **Dynamic route registration:** Allows plugins to register and remove HTTP routes at runtime.
//! - **WebSocket server and client setup:** Enables real-time communication between engine, plugins, and web UI.
//! - **REST API request routing:** Handles API requests and dispatches them to plugins.
//! - **Static file serving:** Serves the web application and plugin static assets.
//!
//! ## Architecture
//! - **Router Manager:** Handles dynamic route registration/removal.
//! - **Plugin Registry:** Manages loaded plugins and their metadata.
//! - **WebSocket System:** Enables real-time communication for status and control messages.
//! - **Execution Plan:** Controls plugin loading sequence and configuration.
//!
//! ## Entry Points
//! - [`start_oobe_server`] - FFI-safe entry point for non-Rust platforms (spawns a new thread).
//! - [`start_server_async`] - Main async entry point for Rust applications (starts the engine).
//!
//! ## Usage
//! The engine is typically started via `start_oobe_server()` (for FFI) or `start_server_async().await` (for Rust).
//! Plugins are loaded according to the execution plan, and the web UI is served at `http://127.0.0.1:8080/`.
//!
//! ## Modules
//! - [`router_manager`] - Dynamic HTTP route management.
//! - [`plugin_manager`] - Plugin loading and registry.
//! - [`websocket_manager`] - WebSocket topics and client/server state.
//!
//! ## Example
//! ```no_run
//! use engine::start_server_async;
//! #[tokio::main]
//! async fn main() {
//!     start_server_async().await;
//! }
//! ```

// Standard library imports
use std::{ net::SocketAddr, sync::{ Arc, Mutex } }; // For network sockets and thread-safe shared state
use std::fs; // For file system operations
use std::path::PathBuf; // For path manipulation
use std::ffi::CString; // For C-compatible strings used in FFI
use std::time::Duration; // For time-based operations
use std::collections::HashMap; // For token cache

// Async runtime imports
use tokio::net::TcpListener; // For asynchronous TCP socket listening

// Web framework imports
use axum::Router; // For HTTP routing
use axum::routing::{ any, get }; // For route handler definitions
use axum::response::Response; // For HTTP responses
use axum::body::Body; // For HTTP body content
use axum::http::StatusCode; // For HTTP status codes

// JWT authentication
use libjwt::routes::create_auth_router_with_cache;
use libjwt::SharedTokenCache;

// Import all the required logging components
use liblogger;
use plugin_core::{log_debug, log_info, log_warn, log_error}; // Logging utilities
use liblogger_macros::*; // Logging macro extensions
use ctor::ctor;

// Initialize logger attributes
initialize_logger_attributes!();

// Use the same initialization pattern that plugins use
#[ctor]
fn on_load() {
    // Initialize the logger for the engine the same way plugins do
    if let Err(e) = plugin_core::init_logger("engine") {
        eprintln!("[engine] Failed to initialize logger: {}", e);
    }
    
    println!("[engine] >>> ENGINE LOADED");
}

// Local module declarations
mod router_manager;
mod websocket_manager;
mod plugin_manager; // Add this line

// Local module imports
use plugin_manager::PluginManager; // Add this line
use router_manager::RouterManager;
use websocket_manager::{
    WS_SUBSCRIBERS,
    ENGINE_WS_CLIENT,
    WELCOME_COMPLETED,
    WIFI_COMPLETED,
    EXECPLAN_COMPLETED,
    PROVISION_COMPLETED,
    NETWORK_CONNECTED,
    SWITCH_ROUTE,
};

// Variable defining route destination after login completion
// static ROUTE_AFTER_LOGIN: Lazy<Mutex<&str>> = Lazy::new(|| Mutex::new("/provision/web"));

// Engine core functionality
use engine_core::{
    plugin_loader::load_plugin,
    plugin_registry::PluginRegistry,
    handlers::dispatch_plugin_api,
    execution_plan_updater::{ ExecutionPlanUpdater, PlanLoadSource },
    execution_plan::ExecutionPlanLoader,
    plugin_metadata::PluginMetadata,
    plugin_utils::prepare_plugin_binary,
};

// Plugin core types
use plugin_core::{ HttpMethod, ApiRequest }; // Remove PluginContext as it's unused

// WebSocket functionality
use libws::handle_socket;
use libws::ws_client::WsClient;

// Custom logger initialization to ensure all logs are displayed
/// Initializes the custom logger for the engine.
/// Attempts to load configuration from `app_config.toml`, falls back to console logging if unavailable.
/// Prints test log messages at all levels.
fn initialize_custom_logger() {
    // Initialize logger with debug threshold to ensure all logs are shown
    match plugin_core::init_logger("engine") {
        Ok(_) => {
            println!("[engine] Logger successfully initialized from config file");
        }
        Err(_e) => {
            println!("[engine] Error initializing logger from config: {}", _e);
            
            // Fall back to console logging by calling init_logger again
            // The plugin_core init_logger will handle fallback internally
            if let Err(e) = plugin_core::init_logger("engine") {
                eprintln!("[engine] Fatal error: Failed to initialize logger: {}", e);
            }
            println!("[engine] Failed to initialize file logger, falling back to console");
        }
    }

    // Print a clear marker to see if logger is working
    log_info!("======== ENGINE LOGGER INITIALIZED ========");
    log_debug!("Debug logging is enabled");
    log_info!("Info logging is enabled");
    log_warn!("Warning logging is enabled");
    log_error!("Error logging is enabled");
    log_info!("======== ENGINE LOGGER TEST COMPLETE ========");
}

//
// WebSocket Client Management
// -------------------------
//

/// Asynchronously subscribes to a WebSocket topic and handles incoming messages.
///
/// This function encapsulates the common logic for subscribing to a topic, logging the subscription,
/// and defining the message handling logic.
///
/// # Arguments
///
/// * `client_arc`: An `Arc<Mutex<WsClient>>` representing the WebSocket client.
/// * `topic`: A string slice representing the topic to subscribe to.
/// * `route`: A string slice representing the route to switch to when a message is received.
async fn subscribe_and_handle(
    client_arc: Arc<Mutex<WsClient>>,
    topic: &'static str,
    route: &'static str
) {
    let client_for_topic = client_arc.clone();
    {
        let mut client = client_arc.lock().unwrap();
        client.subscribe("engine_subscriber", topic, "").await;
        log_debug!(
            format!("Engine, subscribed to topic: {}, will handle route: {}", topic, route).as_str()
        );

        client.on_message(topic, move |_msg| {
            log_debug!(format!("[engine] => {}: received", topic).as_str());

            // Call the reusable function
            tokio::spawn(
                publish_ws_message(client_for_topic.clone(), "engine", SWITCH_ROUTE, route)
            );
        });
    }
}

/// Creates and initializes the WebSocket client for the engine.
/// Sets up subscriptions for status changes and route switching.
/// This should be called once at startup.
pub async fn create_ws_engine_client() {
    log_debug!("Creating ws client for the engine");
    let url = "ws://127.0.0.1:8081/ws";

    // Connect to the WebSocket server with retries.
    let mut retries = 0;
    const MAX_RETRIES: u8 = 5;
    let mut client = None;

    while retries < MAX_RETRIES {
        match WsClient::connect("engine", url).await {
            Ok(connected_client) => {
                client = Some(connected_client);
                break;
            }
            Err(_err) => {
                retries += 1;
                // Actually use the error variable by directly printing it with its display implementation
                log_debug!(format!("Failed to connect WsClient (attempt {}/{}): {}", retries, MAX_RETRIES, _err).as_str());
                if retries < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    // Check if we successfully connected after retries
    let client = match client {
        Some(c) => c,
        None => {
            log_error!(
                format!("Failed to connect to WebSocket server after {} attempts. Exiting.", MAX_RETRIES).as_str()
            );
            return;
        }
    };

    // Store the WebSocket client in the static variable.
    if ENGINE_WS_CLIENT.set(Arc::new(Mutex::new(client))).is_err() {
        log_debug!("Failed to set ENGINE_WS_CLIENT: already initialized");
        return;
    }

    log_debug!("ws client for the engine created");

    // Subscribe to SWITCH_ROUTE topic
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        let mut client = client_arc.lock().unwrap();
        client.subscribe("engine_subscriber", SWITCH_ROUTE, "").await;
        log_debug!("Engine, subscribed to SWITCH_ROUTE");

        client.on_message(SWITCH_ROUTE, |_msg| {
            log_debug!(format!("[engine] => SWITCH_ROUTE: {}", _msg).as_str());
        });
    }

    // Subscribe to WELCOME_COMPLETED topic
    // Route next to /wifi/web
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle(client_arc.clone(), WELCOME_COMPLETED, "/wifi/web").await;
    }

    // Subscribe to WIFI_COMPLETED topic
    // Route next to /execution/web
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle(client_arc.clone(), WIFI_COMPLETED, "/execution/web").await;
    }

    // Subscribe to EXECPLAN_COMPLETED topic
    // Route next to /login/web
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle(client_arc.clone(), EXECPLAN_COMPLETED, "/login/web").await;
    }

    // Subscribe to LOGIN_COMPLETED topic
    // This is an entry point for a PL specific plugin
    // Route next to a dynamic route -- default is /provision/web
    /*
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        let client_for_login = client_arc.clone();
        {
            let mut client = client_arc.lock().unwrap();
            client.subscribe("engine_subscriber", LOGIN_COMPLETED, "").await;
            log_debug!("Engine, subscribed to LOGIN_COMPLETED");

            client.on_message(LOGIN_COMPLETED, move |msg| {
                log_debug!(format!("[engine] => LOGIN_COMPLETED: {}", msg).as_str());

                // Call the reusable function
                tokio::spawn(
                    publish_ws_message(
                        client_for_login.clone(),
                        "engine",
                        SWITCH_ROUTE,
                        *ROUTE_AFTER_LOGIN.lock().unwrap()
                    )
                );
            });
        }
    }
    */

    // Important Note -- User logged in and we handed off to plugin_settings (loaded dynamically through the execution plan)

    // Subscribe to PROVISION_COMPLETED topic
    // Route next to /status/web
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle(client_arc.clone(), PROVISION_COMPLETED, "/status/web").await;
    }

}

/// Utility function to publish a message to a websocket topic using a client.
/// Handles locking, timestamp, and logging.
///
/// # Arguments
/// * `client_arc` - Arc-wrapped Mutex of the WebSocket client.
/// * `client_name` - Name of the client (e.g., "engine").
/// * `topic_name` - WebSocket topic to publish to.
/// * `payload` - Message payload (as string).
pub async fn publish_ws_message(
    client_arc: Arc<Mutex<WsClient>>,
    client_name: &str,
    topic_name: &str,
    payload: &str
) {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let client_name = client_name.to_string();
    let topic_name = topic_name.to_string();
    let payload = payload.to_string();

    // Use spawn_blocking to avoid Send issues with MutexGuard
    tokio::task
        ::spawn_blocking(move || {
            if let Ok(mut client) = client_arc.lock() {
                let rt = tokio::runtime::Handle::current();
                let _ = rt.block_on(
                    client.publish(&client_name, &topic_name, &payload, &timestamp)
                );
                log_debug!(
                    format!(
                        "engine, published {} '{}' to topic '{}'",
                        client_name,
                        payload,
                        topic_name
                    ).as_str()
                );
            }
        }).await
        .ok();
}

//
// Execution Plan Management
// -----------------------
//

/// Loads and processes the execution plan that controls plugin loading.
/// Supports both remote and local fallback plans.
/// Returns the plan source and a vector of plugin metadata if successful.
//#[log_entry_exit]
//#[measure_time]
pub async fn run_exection_plan_updater() -> Option<(PlanLoadSource, Vec<PluginMetadata>)> {
    let local_path = "execution_plan.toml";

    match ExecutionPlanUpdater::fetch_and_prepare_latest(local_path) {
        Ok(plan_status) => {
            let plan_path = match &plan_status {
                PlanLoadSource::Remote(path) => path,
                PlanLoadSource::LocalFallback(path) => path,
            };

            match ExecutionPlanLoader::load_from_file(plan_path) {
                Ok(plan) => {
                    let _plan_type = match plan_status {
                        PlanLoadSource::Remote(_) => "remote",
                        PlanLoadSource::LocalFallback(_) => "local fallback",
                    };

                    log_debug!(
                        format!(
                            "Execution plan [{}] loaded with {} plugins",
                            _plan_type,
                            plan.plugins.len()
                        ).as_str()
                    );

                    log_debug!(
                        format!(
                            "Execution plan [{}] loaded with {} handoffs",
                            _plan_type,
                            plan.handoffs.handoff_events.len()
                        ).as_str()
                    );

                    for event in &plan.handoffs.handoff_events {
                        println!("Engine: Setting up handoff event: {}", event);
                    }

                    // Dynamic plugins are displayed based on the value of `run_after_event_name` defined in execution_plan.toml.
                    // Multiple dynamic plugins can be chained together to run sequentially.
                    // When control needs to transition from a dynamic plugin to a core plugin (i.e., a built-in engine plugin),
                    // we use the list of handoff events from the TOML file to determine the transition point.
                    // Each handoff event is matched manually to the corresponding core plugin that should be executed next.

                    // First one in the list is for the provisioning plugin
                    if plan.handoffs.handoff_events.len() > 0 {
                        if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
                            let handoff_event = Box::leak(
                                plan.handoffs.handoff_events[0].clone().into_boxed_str()
                            );
                            subscribe_and_handle(
                                client_arc.clone(),
                                handoff_event,
                                "/provision/web"
                            ).await;
                        }
                    }

                    // Second one in the list is for the finish plugin
                    if plan.handoffs.handoff_events.len() > 1 {
                        if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
                            let handoff_event = Box::leak(
                                plan.handoffs.handoff_events[1].clone().into_boxed_str()
                            );
                            subscribe_and_handle(
                                client_arc.clone(),
                                handoff_event,
                                "/finish/web"
                            ).await;
                        }
                    }

                    // Log details for each plugin in the execution plan
                    for (idx, plugin) in plan.plugins.iter().enumerate() {
                        let run_after_event_name = plugin.run_after_event_name
                            .as_deref()
                            .unwrap_or("None");
                        let completed_event_name = plugin.completed_event_name
                            .as_deref()
                            .unwrap_or("None");
                        let plugin_description = if plugin.plugin_description.is_empty() {
                            "None"
                        } else {
                            &plugin.plugin_description
                        };

                        let _plugin_details = format!(
                            "Plugin[{}] Details:\n  name: {}\n  route: {}\n  version: {}\n  location_type: {}\n  base_path: {}\n  team: {}\n  eng_contact: {}\n  ops_contact: {}\n  run_async: {}\n  visible_in_ui: {}\n  description: {}\n  run_after_event: {}\n  completed_event: {}",
                            idx,
                            plugin.name,
                            plugin.plugin_route,
                            plugin.version,
                            plugin.plugin_location_type,
                            plugin.plugin_base_path,
                            plugin.team_name,
                            plugin.engineering_contact_email,
                            plugin.operation_contact_email,
                            plugin.run_async,
                            plugin.visible_in_ui,
                            plugin_description,
                            run_after_event_name,
                            completed_event_name
                        );

                        log_debug!(_plugin_details.as_str());

                        // This code determines which plugin should be loaded after the login event
                        // We search for plugins configured to run after LoginCompleted
                        // The selected plugin's route will be used for navigation after login
                        // We hardocded the LoginCompleted event becuase it is a system event
                        /*
                        if run_after_event_name == "LoginCompleted" {
                            log_debug!(
                                format!(
                                    "Plugin name: {} - LoginCompleted event found in execution plan",
                                    plugin.name.clone()
                                ).as_str()
                            );
                            *ROUTE_AFTER_LOGIN.lock().unwrap() = Box::leak(
                                (plugin.plugin_route.clone() + "/web").into_boxed_str()
                            );
                        } else {
                         */
                        let plugin_route = plugin.plugin_route.clone();
                        let run_after_event_name_owned = Box::leak(
                            Box::new(run_after_event_name.to_string())
                        ).as_str();
                        let route = Box::leak(format!("{}/web", plugin_route).into_boxed_str());

                        log_debug!(
                            format!(
                                "Plugin name: {} - run_after_event_name: {}, route: {} from execution plan",
                                plugin.name.clone(),
                                run_after_event_name,
                                route
                            ).as_str()
                        );

                        if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
                            subscribe_and_handle(
                                client_arc.clone(),
                                run_after_event_name_owned,
                                route
                            ).await;
                        }

                        // To add a new route at runtime:
                        RouterManager::add_plugin_route(&plugin_route, route).await;

                        // To add a static route at runtime:
                        // RouterManager::add_static_route("/docs", "documentation").await;
                        //}
                    }

                    Some((plan_status, plan.plugins))
                }
                Err(_e) => {
                    log_debug!(
                        format!("Failed to parse execution plan: {}", _e.to_string()).as_str()
                    );
                    None
                }
            }
        }
        Err(_e) => {
            log_debug!(format!("Failed to resolve execution plan: {}", _e.to_string()).as_str());
            None
        }
    }
}

/// Loads a plugin from the given path and registers it with the engine.
/// Stores the plugin library to prevent premature unloading.
///
/// # Arguments
/// * `path` - Path to the plugin binary.
/// * `registry` - Shared plugin registry.
/// * `lib_holder` - Vector holding loaded plugin libraries.
// #[measure_time]
fn load_and_register(
    path: PathBuf,
    registry: &Arc<PluginRegistry>,
    lib_holder: &mut Vec<libloading::Library>
) {
    match load_plugin(&path) {
        Ok((plugin, lib)) => {
            registry.register(plugin);
            lib_holder.push(lib); // retain library to avoid drop
        }
        Err(_e) => {
            log_debug!(format!("Failed to load plugin from {}: {}", path.display(), _e).as_str()); 
        }
    }
}

//
// Engine Entry Points
// ----------------
//

/// FFI-safe entry point for non-Rust platforms.
/// Spawns the engine in a new thread with its own runtime.
/// This is the recommended entry point for C/C++ or other FFI consumers.
#[no_mangle]
pub extern "C" fn start_oobe_server() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });
}

/// Main async entry point for Rust applications.
/// Initializes all engine components and starts the server.
/// This function should be called from a Tokio runtime.
pub async fn start_server_async() {
    initialize_custom_logger();

    // WebSocket Server Initialization
    tokio::spawn({
        let subs = WS_SUBSCRIBERS.clone();
        async move {
            use axum::{ Router, routing::get };
            use axum::extract::connect_info::ConnectInfo;

            let ws_app = Router::new().route(
                "/ws",
                get(move |ws, ConnectInfo(addr)| {
                    handle_socket(ws, ConnectInfo(addr), subs.clone())
                })
            );

            let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
            log_debug!("[engine] WebSocket server listening at ws://127.0.0.1:8081/ws");

            axum::serve(
                listener,
                ws_app.into_make_service_with_connect_info::<SocketAddr>()
            ).await.unwrap();
        }
    });

    // WebSocket Client Initialization
    create_ws_engine_client().await;

    // Plugin Registry Initialization
    let registry = Arc::new(PluginRegistry::new());
    let mut plugin_libraries = Vec::new();
    let mut plugin_manager = PluginManager::new(registry.clone());

    // Core Plugin Loading
    // ("plugin_status", "statusmessage=none"),
    // ("plugin_task_agent_headless", "runworkflow=false"),
    let plugins_to_load = [
        ("plugin_welcome", "continue=false"),
        ("plugin_wifi", "connected=false"),
        ("plugin_execplan", "hasupdate=true"),
        ("plugin_login", "isloggedin=false"),
        ("plugin_provisioning", "isprovisioned=false"),
        ("plugin_terms", "accepted=false"),
        ("plugin_finish", "done=true"),
    ];

    for (plugin_name, params) in plugins_to_load {
        log_debug!(format!("Loading the {} plugin", plugin_name).as_str());

        if let Some(plugin) = plugin_manager.load_plugin(plugin_name, params) {
            log_debug!(format!("Registered {}", plugin_name).as_str());

            // Special handling for task_agent_headless post-load setup
            if plugin_name == "plugin_task_agent_headless" {
                let task_agent = plugin.clone();
                if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
                    let mut client = client_arc.lock().unwrap();

                    client.subscribe("engine_subscriber", NETWORK_CONNECTED, "").await;
                    log_debug!("Engine, subscribed to NETWORK_CONNECTED");

                    client.on_message(NETWORK_CONNECTED, move |_msg| {
                        log_debug!(format!("[engine] => NETWORK_CONNECTED: {}", _msg).as_str());

                        if let Some(run_workflow_fn) = task_agent.run_workflow {
                            let json_bytes = r#"{"task": "background_job"}"#.as_bytes().to_vec();
                            let json_len = json_bytes.len();
                            let body_ptr = Box::into_raw(
                                json_bytes.into_boxed_slice()
                            ) as *const u8;

                            let request = ApiRequest {
                                method: HttpMethod::Post,
                                path: CString::new("job").unwrap().into_raw(),
                                headers: std::ptr::null(),
                                header_count: 0,
                                body_ptr,
                                body_len: json_len,
                                content_type: std::ptr::null(),
                                query: std::ptr::null(),
                            };

                            let _ = run_workflow_fn(&request);
                        }
                    });
                }
            }
        } else {
            log_debug!(format!("Failed to load {}", plugin_name).as_str());
            return;
        }
    }

    log_debug!("Core plugins loaded", None);

    // Move plugin libraries to holder
    plugin_libraries.extend(plugin_manager.get_plugin_libraries().drain(..));

    log_debug!("Loading the execution plan");

    let Some((plan_status, plugins)) = run_exection_plan_updater().await else {
        log_debug!("Execution plan loading failed. Cannot continue.");
        return;
    };

    let allow_write = matches!(plan_status, PlanLoadSource::Remote(_));

    // Modify the error handler for plugin preparation
    for plugin_meta in plugins {
        match prepare_plugin_binary(&plugin_meta, allow_write) {
            Ok(local_path) => load_and_register(local_path, &registry, &mut plugin_libraries),
            Err(_e) => {
                let _source = match plan_status {
                    PlanLoadSource::Remote(_) => "remote plan",
                    PlanLoadSource::LocalFallback(_) => "local fallback plan",
                };

                log_debug!(
                    format!(
                        "[WARN] Plugin '{}' failed to prepare from '{}' ({}): {}",
                        plugin_meta.name,
                        plugin_meta.plugin_location_type,
                        _source,
                        _e
                    ).as_str()
                );
            }
        }
    }

    // JWT Authentication Setup - START
    log_debug!("********** JWT AUTHENTICATION SETUP - BEGIN **********");
    
    // Create a shared token cache for JWT authentication
    let token_cache: SharedTokenCache = Arc::new(Mutex::new(HashMap::new()));
    log_debug!("JWT Auth: Token cache created successfully");
    
    // Create authentication routes
    log_debug!("JWT Auth: Creating authentication router...");
    let auth_router = create_auth_router_with_cache(token_cache.clone());
    log_debug!("JWT Auth: Authentication router created successfully");

    log_debug!("JWT Auth: Creating base router...");
    let mut base_router = Router::new();
    log_debug!("JWT Auth: Base router created successfully");

    // Merge authentication router with base router
    log_debug!("JWT Auth: Merging authentication router with base router...");
    base_router = base_router.merge(auth_router);
    log_debug!("JWT Auth: Authentication router merged successfully");
    
    log_debug!("********** JWT AUTHENTICATION SETUP - COMPLETE **********");

    // Setup API routes
    log_debug!("Setting up API routes...");
    let plugin_api_router = Router::new().route(
        "/:plugin/:resource",
        any(dispatch_plugin_api).with_state(registry.clone())
    );
    base_router = base_router.nest("/api", plugin_api_router);

    // Initialize router manager with base routes
    {
        let mut router = RouterManager::get_manager().write().unwrap();
        *router = base_router;
    }

    // Register initial plugin routes
    for plugin in registry.all() {
        RouterManager::add_plugin_route(&plugin.plugin_route, &plugin.static_path).await;
    }

    // Add webapp route
    RouterManager::add_static_route("/", "webapp").await;

    // Example of runtime route management:
    /*
    // To add a new route at runtime:
    RouterManager::add_plugin_route("settings", "settings/web").await;
    
    // To add a static route at runtime:
    RouterManager::add_static_route("/docs", "documentation").await;
    */

    async fn fallback_handler() -> Response {
        match fs::read_to_string("webapp/index.html") {
            Ok(content) =>
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Body::from(content))
                    .unwrap(),
            Err(_) =>
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("index.html not found"))
                    .unwrap(),
        }
    }

    // Get the router for serving
    let app = {
        let router = RouterManager::get_manager().write().unwrap();
        router.clone().fallback(get(fallback_handler))
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    log_debug!(format!("Listening at http://{}", addr).as_str());

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
