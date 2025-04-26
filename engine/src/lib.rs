//! Engine Core Library
//! 
//! This module serves as the main entry point for the plugin-based OOBE engine.
//! It handles:
//! - Plugin loading and lifecycle management
//! - Dynamic route registration 
//! - WebSocket server and client setup
//! - REST API request routing
//! - Static file serving
//!
//! # Architecture
//! 
//! The engine uses a few key components:
//! - Router Manager - Handles dynamic route registration/removal
//! - Plugin Registry - Manages loaded plugins
//! - WebSocket System - Enables real-time communication
//! - Execution Plan - Controls plugin loading sequence

// Standard library imports
use std::{ net::SocketAddr, sync::{ Arc, Mutex } };
use std::fs;
use std::path::PathBuf;
use std::ffi::CString;

// Async runtime imports
use tokio::net::TcpListener;

// Web framework imports
use axum::Router;
use axum::routing::{any, get};
use axum::response::Response;
use axum::body::Body;
use axum::http::StatusCode;

// Logger imports
use logger::{ LoggerLoader, LogLevel };

// Local module declarations
mod router_manager;
mod websocket_manager;
mod plugin_manager; // Add this line

// Local module imports
use plugin_manager::PluginManager; // Add this line
use router_manager::RouterManager;
use websocket_manager::{
    WS_SUBSCRIBERS, ENGINE_WS_CLIENT,
    STATUS_CHANGED, NETWORK_CONNECTED, SWITCH_ROUTE
};

// Engine core functionality
use engine_core::{
    plugin_loader::load_plugin,
    plugin_registry::PluginRegistry,
    handlers::dispatch_plugin_api,
    execution_plan_updater::{ExecutionPlanUpdater, PlanLoadSource},
    execution_plan::ExecutionPlanLoader,
    plugin_metadata::PluginMetadata,
    plugin_utils::prepare_plugin_binary,
};

// Plugin core types
use plugin_core::{HttpMethod, ApiRequest}; // Remove PluginContext as it's unused

// WebSocket functionality
use ws_server::handle_socket;
use ws_server::ws_client::WsClient;

//
// WebSocket Client Management
// -------------------------
//

/// Creates and initializes the WebSocket client for the engine.
/// Sets up subscriptions for status changes and route switching.
pub async fn create_ws_engine_client() {
    println!("Creating ws client for the engine");
    let url = "ws://127.0.0.1:8081/ws";

    // Connect to the WebSocket server.
    let client = WsClient::connect("engine", url)
        .await
        .expect("Failed to connect WsClient");

    // Store the WebSocket client in the static variable.
    if ENGINE_WS_CLIENT.set(Arc::new(Mutex::new(client))).is_err() {
        eprintln!("Failed to set ENGINE_WS_CLIENT: already initialized");
        return;
    }

    println!("ws client for the engine created");

    // Subscribe to SWITCH_ROUTE topic
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        let mut client = client_arc.lock().unwrap();
        client.subscribe("engine_subscriber", SWITCH_ROUTE, "").await;
        println!("Engine, subscribed to SWITCH_ROUTE");

        client.on_message(SWITCH_ROUTE, |msg| {
            println!("[engine] => SWITCH_ROUTE: {}", msg);
        });
    }

    // Subscribe to STATUS_CHANGED topic
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        let mut client = client_arc.lock().unwrap();
        client.subscribe("engine_subscriber", STATUS_CHANGED, "").await;
        println!("Engine, subscribed to STATUS_CHANGED");

        client.on_message(STATUS_CHANGED, |msg| {
            println!("[engine] => STATUS_CHANGED: {}", msg);
        });
    }
}

//
// Execution Plan Management  
// -----------------------
//

/// Loads and processes the execution plan that controls plugin loading.
/// Supports both remote and local fallback plans.
pub fn run_exection_plan_updater() -> Option<(PlanLoadSource, Vec<PluginMetadata>)> {
    let local_path = "execution_plan.toml";

    match ExecutionPlanUpdater::fetch_and_prepare_latest(local_path) {
        Ok(plan_status) => {
            let plan_path = match &plan_status {
                PlanLoadSource::Remote(path) => path,
                PlanLoadSource::LocalFallback(path) => path,
            };

            match ExecutionPlanLoader::load_from_file(plan_path) {
                Ok(plan) => {
                    println!(
                        "Execution plan [{}] loaded with {} plugins",
                        match plan_status {
                            PlanLoadSource::Remote(_) => "remote",
                            PlanLoadSource::LocalFallback(_) => "local fallback",
                        },
                        plan.plugins.len()
                    );
                    Some((plan_status, plan.plugins))
                }
                Err(e) => {
                    eprintln!("Failed to parse execution plan: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to resolve execution plan: {}", e);
            None
        }
    }
}

//
// Plugin Management
// ---------------
//

/// Loads a plugin from the given path and registers it with the engine.
/// Stores the plugin library to prevent premature unloading.
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
        Err(e) => eprintln!("Failed to load plugin from {}: {}", path.display(), e),
    }
}

//
// Engine Entry Points
// ----------------
//

/// FFI-safe entry point for non-Rust platforms.
/// Spawns the engine in a new thread with its own runtime.
#[no_mangle]
pub extern "C" fn start_oobe_server() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });
}

/// Main async entry point for Rust applications.
/// Initializes all engine components and starts the server.
pub async fn start_server_async() {
    // Logger Initialization
    LoggerLoader::init("app_config.toml").await;
    let logger = LoggerLoader::get_logger();
    logger.log(LogLevel::Info, "Logger initialized");
    logger.log(LogLevel::Info, "Creating plugin registry");

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
            println!("[engine] WebSocket server listening at ws://127.0.0.1:8081/ws");

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
    let plugins_to_load = [
        ("plugin_terms", "accepted=false"),
        ("plugin_wifi", "connected=false"),
        ("plugin_status", "statusmessage=none"),
        ("plugin_task_agent_headless", "runworkflow=false"),
    ];

    for (plugin_name, config) in plugins_to_load {
        logger.log(LogLevel::Info, &format!("Loading the {} plugin", plugin_name));
        
        if let Some(plugin) = plugin_manager.load_plugin(plugin_name, config) {
            logger.log(LogLevel::Info, &format!("Registered {}", plugin_name));
            
            // Special handling for task_agent_headless post-load setup
            if plugin_name == "plugin_task_agent_headless" {
                let task_agent = plugin.clone();
                if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
                    let mut client = client_arc.lock().unwrap();
                    
                    client.subscribe("engine_subscriber", NETWORK_CONNECTED, "").await;
                    println!("Engine, subscribed to NETWORK_CONNECTED");

                    client.on_message(NETWORK_CONNECTED, move |msg| {
                        println!("[engine] => NETWORK_CONNECTED: {}", msg);
                        
                        if let Some(run_workflow_fn) = task_agent.run_workflow {
                            let json_bytes = r#"{"task": "background_job"}"#.as_bytes().to_vec();
                            let json_len = json_bytes.len();
                            let body_ptr = Box::into_raw(json_bytes.into_boxed_slice()) as *const u8;
                            
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
            logger.log(LogLevel::Error, &format!("Failed to load {}", plugin_name));
            return;
        }
    }

    logger.log(LogLevel::Info, "Core plugins loaded");

    // Move plugin libraries to holder
    plugin_libraries.extend(plugin_manager.get_plugin_libraries().drain(..));

    let logger = LoggerLoader::get_logger();

    logger.log(LogLevel::Info, "Loading the execution plan");

    let Some((plan_status, plugins)) = run_exection_plan_updater() else {
        eprintln!("Execution plan loading failed. Cannot continue.");
        return;
    };

    let allow_write = matches!(plan_status, PlanLoadSource::Remote(_));

    for plugin_meta in plugins {
        match prepare_plugin_binary(&plugin_meta, allow_write) {
            Ok(local_path) => load_and_register(local_path, &registry, &mut plugin_libraries),
            Err(e) => {
                let source = match plan_status {
                    PlanLoadSource::Remote(_) => "remote plan",
                    PlanLoadSource::LocalFallback(_) => "local fallback plan",
                };

                eprintln!(
                    "[WARN] Plugin '{}' failed to prepare from '{}' ({}): {}",
                    plugin_meta.name,
                    plugin_meta.plugin_location_type,
                    source,
                    e
                );
            }
        }
    }

    let mut base_router = Router::new();

    // Setup API routes
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
    println!("Listening at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
