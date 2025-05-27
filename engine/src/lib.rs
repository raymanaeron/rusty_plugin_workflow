//
// OOBE Engine - Main library file
//

// ===== Standard library imports =====
use std::{ net::SocketAddr, sync::{ Arc, Mutex } }; // For network sockets and thread-safe shared state
use std::fs; // For file system operations
use std::path::PathBuf; // For path manipulation
use std::ffi::CString; // For C-compatible strings used in FFI
use std::time::Duration; // For time-based operations
use std::sync::atomic::{ AtomicPtr, Ordering }; // For atomic operations

// ===== Async runtime imports =====
use tokio::net::TcpListener; // For asynchronous TCP socket listening

// ===== Web framework imports =====
use axum::Router; // For HTTP routing
use axum::routing::{ any }; // For route handler definitions
use axum::response::Response; // For HTTP responses
use axum::body::Body; // For HTTP body content
use axum::http::StatusCode; // For HTTP status codes
use axum::http::{ Method, header };
use tower_http::cors::{ Any, CorsLayer }; // For CORS support
use tower_http::trace::TraceLayer; // For HTTP request tracing

// ===== Authentication =====
use libjwt::{ JwtManager, create_auth_router_with_cache }; // JWT authentication

// ===== Logging =====
use liblogger;
use plugin_core::{ log_debug, log_info, log_warn, log_error }; // Logging utilities
use liblogger_macros::*; // Logging macro extensions
use ctor::ctor; // Constructor attribute for initialization

// ===== Local module declarations =====
mod router_manager;
mod websocket_manager;
mod plugin_manager;

// ===== Local module imports =====
use plugin_manager::PluginManager;
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

// ===== Engine core functionality =====
// jwt_gen_util::get_jwt_token,
use engine_core::{
    plugin_loader::load_plugin,
    plugin_registry::PluginRegistry,
    handlers::dispatch_plugin_api,
    execution_plan_updater::{ ExecutionPlanUpdater, PlanLoadSource },
    execution_plan::ExecutionPlanLoader,
    plugin_metadata::PluginMetadata,
    plugin_utils::prepare_plugin_binary,
};

// ===== Plugin core types =====
use plugin_core::{ HttpMethod, ApiRequest };

// ===== WebSocket functionality =====
use libws::handle_socket;
use libws::ws_client::WsClient;

// ===== Global variables =====
// Registry pointer to maintain plugins across the application lifetime
static REGISTRY_PTR: AtomicPtr<Arc<PluginRegistry>> = AtomicPtr::new(std::ptr::null_mut());
// Plugin libraries pointer to prevent dynamic libraries from being unloaded
static PLUGIN_LIBRARIES_PTR: AtomicPtr<Vec<libloading::Library>> = AtomicPtr::new(
    std::ptr::null_mut()
);

// Initialize logger attributes
initialize_logger_attributes!();

//
// ===== Initialization Functions =====
//

// Engine initialization called at load time
#[ctor]
fn on_load() {
    // Initialize the logger for the engine the same way plugins do
    if let Err(e) = plugin_core::init_logger("engine") {
        eprintln!("[engine] Failed to initialize logger: {}", e);
    }

    println!("[engine] >>> ENGINE LOADED");
}

// Initializes and tests the custom logger
fn initialize_custom_logger() {
    // The logger is already initialized in the on_load() ctor function,
    // so we just need to print some test messages here

    // Print a clear marker to see if logger is working
    log_info!("======== ENGINE LOGGER INITIALIZED ========");
    log_debug!("Debug logging is enabled");
    log_info!("Info logging is enabled");
    log_warn!("Warning logging is enabled");
    log_error!("Error logging is enabled");
    log_info!("======== ENGINE LOGGER TEST COMPLETE ========");
}

//
// ===== WebSocket Communication Functions =====
//

// Creates and initializes the WebSocket client for the engine
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
                log_debug!(
                    format!(
                        "Failed to connect WsClient (attempt {}/{}): {}",
                        retries,
                        MAX_RETRIES,
                        _err
                    ).as_str()
                );
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
        // subscribe_and_handle(client_arc.clone(), WELCOME_COMPLETED, "/wifi/web").await;
        subscribe_and_handle(client_arc.clone(), WELCOME_COMPLETED, "/mwifi/web").await;
    }

    // Subscribe to WIFI_COMPLETED topic
    // Route next to /execution/web
    // Fix: Initialize registry and plugin_libraries variables before using them
    let registry = Arc::new(PluginRegistry::new());
    let plugin_libraries = Arc::new(Mutex::new(Vec::new()));

    // Fix: Access client_arc from ENGINE_WS_CLIENT
    // Execution plan plugins are loaded dynamically through the execution plan
    // We check and download a new execution plan if available right after the network connection is established
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle_with_registry(
            client_arc.clone(),
            WIFI_COMPLETED,
            "/execution/web",
            registry.clone(),
            plugin_libraries.clone()
        ).await;
    }

    // Subscribe to EXECPLAN_COMPLETED topic
    // Route next to /login/web
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle(client_arc.clone(), EXECPLAN_COMPLETED, "/login/web").await;
    }

    // Important Note -- User logged in and we handed off to plugin_settings (loaded dynamically through the execution plan)

    // Subscribe to PROVISION_COMPLETED topic
    // Route next to /status/web
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        subscribe_and_handle(client_arc.clone(), PROVISION_COMPLETED, "/status/web").await;
    }
}

// Helper function for subscribing to WebSocket topics and handling navigation
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

// Helper function for subscribing to WebSocket topics with registry handling for dynamic plugins
async fn subscribe_and_handle_with_registry(
    client_arc: Arc<Mutex<WsClient>>,
    topic: &'static str,
    _route: &'static str,
    registry: Arc<PluginRegistry>,
    plugin_libraries: Arc<Mutex<Vec<libloading::Library>>>
) {
    // First, set up the message handler directly without holding locks across await points
    {
        let _client_for_message = client_arc.clone();

        // Make sure to lock and subscribe in a contained scope
        {
            let mut client = client_arc.lock().unwrap();
            client.subscribe("engine", topic, "").await;
            log_debug!(format!("Engine, subscribed to topic: {}", topic).as_str());
        }

        // Create clones of necessary objects before entering the message handler
        let registry_clone = registry.clone();
        let plugin_libraries_clone = plugin_libraries.clone();

        // Set up a separate message handler for WIFI_COMPLETED
        if topic == WIFI_COMPLETED {
            // Use a simpler approach that doesn't involve nested threads
            let msg_client_arc = client_arc.clone();

            // Acquire the lock inside a scope to set up the message handler
            if let Ok(mut client) = client_arc.lock() {
                client.on_message(topic, move |msg| {
                    log_debug!(format!("[engine] => {}: {}", topic, msg).as_str());

                    // Create a dedicated thread for handling this specific message occurrence
                    // This avoids crossing thread boundaries with mutexes
                    let registry_for_thread = registry_clone.clone();
                    let libs_for_thread = plugin_libraries_clone.clone();
                    let client_for_thread = msg_client_arc.clone();

                    // Spawn a standard thread instead of using tokio::spawn
                    std::thread::spawn(move || {
                        // Create a new runtime for this thread
                        let rt = tokio::runtime::Builder
                            ::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("Failed to create runtime for plugin loading");

                        // Execute the async block in the runtime
                        rt.block_on(async {
                            // Create a new Vec to hold the libraries
                            let mut new_libraries = Vec::new();

                            // Load the plugins
                            if
                                !load_execution_plan_plugins(
                                    &registry_for_thread,
                                    &mut new_libraries
                                ).await
                            {
                                log_error!("Failed to load plugins from execution plan");
                                return;
                            }

                            // Update the original vector with the new libraries
                            if let Ok(mut guard) = libs_for_thread.lock() {
                                guard.extend(new_libraries.into_iter());
                            }

                            // Send the navigation message
                            publish_ws_message(
                                client_for_thread,
                                "engine",
                                SWITCH_ROUTE,
                                "/execution/web"
                            ).await;
                        });
                    });
                });
            }
        } else {
            // For other topics, just set up a simple message handler
            if let Ok(mut client) = client_arc.lock() {
                client.on_message(topic, move |msg| {
                    log_debug!(format!("[engine] => {}: {}", topic, msg).as_str());
                });
            }
        }
    }
}

// Publishes a WebSocket message to a specified topic
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
// ===== Execution Plan Management =====
//

// Loads and processes the execution plan that controls plugin loading
pub async fn run_exection_plan_updater() -> Option<(PlanLoadSource, Vec<PluginMetadata>)> {
    let local_path = "execution_plan.toml";

    // Move the Error-returning code inside this function to avoid Send issues
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

//
// ===== Plugin Management =====
//

// Loads and registers a plugin from the given path
#[measure_time]
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

// Loads plugins defined in the execution plan
async fn load_execution_plan_plugins(
    registry: &Arc<PluginRegistry>,
    plugin_libraries: &mut Vec<libloading::Library>
) -> bool {
    log_debug!("Loading the execution plan");

    let Some((plan_status, plugins)) = run_exection_plan_updater().await else {
        log_debug!("Execution plan loading failed. Cannot continue.");
        return false;
    };

    let allow_write = matches!(plan_status, PlanLoadSource::Remote(_));

    // Modify the error handler for plugin preparation
    for plugin_meta in plugins {
        match prepare_plugin_binary(&plugin_meta, allow_write) {
            Ok(local_path) => load_and_register(local_path, &registry, plugin_libraries),
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

    log_debug!("Execution plan load completed");
    true
}

//
// ===== Server Entry Points =====
//

// FFI-safe entry point for non-Rust platforms
#[no_mangle]
pub extern "C" fn start_oobe_server() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });
}

// Main async entry point for Rust applications
pub async fn start_server_async() {
    initialize_custom_logger();

    // WebSocket Server Initialization
    tokio::spawn({
        let subs = WS_SUBSCRIBERS.clone();
        async move {
            use axum::{ Router, routing::get };
            use axum::extract::connect_info::ConnectInfo;
            
            // This should not be here
            /*
            let token = get_jwt_token().await.expect("Failed to get JWT token");
            log_debug!(format!("[engine] => JWT token: {}", token).as_str());
            */

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
    REGISTRY_PTR.store(Box::into_raw(Box::new(registry.clone())), Ordering::Relaxed);

    // Initialize plugin_libraries as Vec type
    let plugin_libraries = Vec::new();
    PLUGIN_LIBRARIES_PTR.store(Box::into_raw(Box::new(plugin_libraries)), Ordering::Relaxed);

    let mut plugin_manager = PluginManager::new(registry.clone());

    // Core Plugin Loading
    let plugins_to_load = [
        ("plugin_welcome", "continue=false"),
        ("plugin_mockwifi", "connected=false"),
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
    let mut plugin_libs = plugin_manager
        .get_plugin_libraries()
        .drain(..)
        .collect::<Vec<_>>();
    unsafe {
        if let Some(libs_ptr) = PLUGIN_LIBRARIES_PTR.load(Ordering::Relaxed).as_mut() {
            libs_ptr.extend(plugin_libs.drain(..));
        }
    }

    // JWT Authentication Setup
    log_debug!("********** JWT AUTHENTICATION SETUP - BEGIN **********");

    // Initialize JWT manager
    let jwt_manager = JwtManager::init().await.expect("Failed to initialize JWT manager");

    // Step 1: Create the base router that will hold everything
    log_debug!("Creating base router...");
    let mut base_router = Router::new();

    // Step 2: Create all API-related routers independently first
    log_debug!("Creating authentication router...");
    let auth_router = create_auth_router_with_cache(jwt_manager.token_cache.clone());

    log_debug!("Creating plugin API router...");
    let plugin_api_router = Router::new().route(
        "/:plugin/:resource",
        any(dispatch_plugin_api).with_state(registry.clone())
    );

    // Step 3: Combine all API routers into a single API router
    log_debug!("Combining all API routers...");
    let api_router = Router::new().merge(auth_router).merge(plugin_api_router);

    // Step 4: Nest the combined API router under /api
    log_debug!("Nesting combined API router under /api path...");
    base_router = base_router.nest("/api", api_router);

    log_debug!("********** JWT AUTHENTICATION SETUP - COMPLETE **********");

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

    // Fallback handler for SPA routing
    #[allow(dead_code)]
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

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(Any);

    // Get the router for serving
    let app = RouterManager::shared_router_service();

    // Apply middleware layers
    let app = app.layer(cors).layer(TraceLayer::new_for_http());

    // Start the HTTP server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    log_debug!(format!("Listening at http://{}", addr).as_str());

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
