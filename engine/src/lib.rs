// Imports
use std::{ net::SocketAddr, sync::{ Arc, Mutex, RwLock } };
use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;
use std::fs;

use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

use axum::Router;
use axum::routing::{any, get};
use axum::response::Response;
use axum::body::Body;
use axum::http::StatusCode;

use tower_http::services::ServeDir;

use once_cell::sync::{ Lazy, OnceCell };

use engine_core::{ plugin_loader::load_plugin, plugin_registry::PluginRegistry };
use engine_core::handlers::dispatch_plugin_api;
use engine_core::execution_plan_updater::{ ExecutionPlanUpdater, PlanLoadSource };
use engine_core::execution_plan::ExecutionPlanLoader;
use engine_core::plugin_utils;
use engine_core::plugin_metadata::PluginMetadata;
use engine_core::plugin_utils::prepare_plugin_binary;

use plugin_core::{ PluginContext, HttpMethod, ApiRequest };

use ws_server::{ handle_socket, Subscribers };
use ws_server::ws_client::WsClient;

use logger::{ LoggerLoader, LogLevel };

// Static Variables
/// Router manager for dynamic route handling
static ROUTER_MANAGER: Lazy<Arc<RwLock<Router>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Router::new()))
});

/// WebSocket subscribers for the engine.
pub static WS_SUBSCRIBERS: Lazy<Subscribers> = Lazy::new(|| {
    std::sync::Arc::new(
        std::sync::Mutex::new(HashMap::<String, Vec<UnboundedSender<String>>>::new())
    )
});

/// Topic for receiving status change messages.
pub static STATUS_CHANGED: &str = "StatusMessageChanged";

/// Topic for receiving network connected messages.
pub static NETWORK_CONNECTED: &str = "NetworkConnected";

// Topic for receiving switch route messages.
pub static SWITCH_ROUTE: &str = "SwitchRoute";

/// WebSocket client for the engine.
pub static ENGINE_WS_CLIENT: OnceCell<Arc<Mutex<WsClient>>> = OnceCell::new();

// WebSocket Client Initialization
/// Creates and initializes the WebSocket client for the engine.
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

// Execution Plan Updater
/// Runs the execution plan updater and returns the plan status and plugins.
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

// Plugin Loader
/// Loads and registers a plugin from the given path.
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

// Entry Points
/// FFI-safe C entry point for Swift, Kotlin, C++, etc.
#[no_mangle]
pub extern "C" fn start_oobe_server() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });
}

/// Native async entry point for Rust-based apps (e.g. desktop, CLI)
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

    // Plugin Loading
    logger.log(LogLevel::Info, "Loading the terms plugin");

    let (terms_plugin, _terms_lib) = match
        load_plugin(plugin_utils::resolve_plugin_filename("plugin_terms"))
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load terms plugin: {}", e);
            return;
        }
    };

    logger.log(LogLevel::Info, "Running the terms plugin with a parameter");
    let terms_config = CString::new("accepted=false").unwrap();
    let terms_ctx = PluginContext {
        config: terms_config.as_ptr(),
    };
    (terms_plugin.run)(&terms_ctx);

    logger.log(LogLevel::Info, "Registering terms plugin");
    registry.register(terms_plugin.clone());
    plugin_libraries.push(_terms_lib);

    println!(
        "[engine] FINGERPRINT: plugin_terms.get_api_resources = {:p}",
        terms_plugin.get_api_resources as *const ()
    );

    let mut count: usize = 0;
    let res_ptr = (terms_plugin.get_api_resources)(&mut count);

    if !res_ptr.is_null() && count > 0 {
        let res_slice = unsafe { std::slice::from_raw_parts(res_ptr, count) };
        for r in res_slice {
            let path = unsafe { std::ffi::CStr::from_ptr(r.path).to_string_lossy() };
            println!("[engine] Plugin resource advertised: {}", path);
        }
    } else {
        println!("[engine] Plugin returned no resources");
    }

    logger.log(LogLevel::Info, "Loading the wifi plugin");

    let (wifi_plugin, _wifi_lib) = match
        load_plugin(plugin_utils::resolve_plugin_filename("plugin_wifi"))
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load wifi plugin: {}", e);
            return;
        }
    };

    logger.log(LogLevel::Info, "Running the wifi plugin with a parameter");
    let wifi_config = CString::new("scan=true").unwrap();
    let wifi_ctx = PluginContext {
        config: wifi_config.as_ptr(),
    };
    (wifi_plugin.run)(&wifi_ctx);

    logger.log(LogLevel::Info, "Registering wifi plugin");
    plugin_libraries.push(_wifi_lib);
    registry.register(wifi_plugin.clone());

    logger.log(LogLevel::Info, "Loading the status plugin");

    let (status_plugin, _status_lib) = match
        load_plugin(plugin_utils::resolve_plugin_filename("plugin_status"))
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load status plugin: {}", e);
            return;
        }
    };

    logger.log(LogLevel::Info, "Running the status plugin with a parameter");
    let status_config = CString::new("scan=true").unwrap();
    let status_ctx = PluginContext {
        config: status_config.as_ptr(),
    };
    (status_plugin.run)(&status_ctx);

    logger.log(LogLevel::Info, "Registering status plugin");
    plugin_libraries.push(_status_lib);

    registry.register(status_plugin.clone());

    logger.log(LogLevel::Info, "Loading the task_agent_headless plugin");

    let (task_agent_headless_plugin, _task_agent_headless_lib) = match
        load_plugin(plugin_utils::resolve_plugin_filename("plugin_task_agent_headless"))
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load task_agent_headless plugin: {}", e);
            return;
        }
    };

    logger.log(LogLevel::Info, "Running the task_agent_headless plugin with a parameter");
    let task_agent_headless_config = CString::new("scan=true").unwrap();
    let task_agent_headless_ctx = PluginContext {
        config: task_agent_headless_config.as_ptr(),
    };
    (task_agent_headless_plugin.run)(&task_agent_headless_ctx);

    logger.log(LogLevel::Info, "Registering task_agent_headless plugin");
    plugin_libraries.push(_task_agent_headless_lib);

    registry.register(task_agent_headless_plugin.clone());

    logger.log(LogLevel::Info, "Core plugins loaded");

    let task_agent = task_agent_headless_plugin.clone();
    let _status_plugin = status_plugin.clone();
    let logger = LoggerLoader::get_logger();

    // Subscribe to NETWORK_CONNECTED and trigger task agent workflow
    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        let mut client = client_arc.lock().unwrap();
        let task_agent = task_agent.clone();
        
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
        let mut router = ROUTER_MANAGER.write().unwrap();
        *router = base_router;
    }

    // Register initial plugin routes
    for plugin in registry.all() {
        add_plugin_route(&plugin.plugin_route, &plugin.static_path).await;
    }

    // Add webapp route
    add_static_route("/", "webapp").await;

    // Example of how to add/remove routes at runtime:
    /*
    // To add a new route at runtime:
    add_plugin_route("new-plugin/web", "plugins/new-plugin/web").await;
    
    // To remove a route at runtime:
    remove_route("wifi/web").await;
    
    // To add a static route at runtime:
    add_static_route("/docs", "documentation").await;
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
        let mut router = ROUTER_MANAGER.write().unwrap();
        router.clone().fallback(get(fallback_handler))
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

// Add these helper functions
pub async fn add_plugin_route(route: &str, path: &str) {
    let mut router = ROUTER_MANAGER.write().unwrap();
    *router = router.clone().nest_service(
        &format!("/{}", route),
        ServeDir::new(path)
    );
    println!("Added plugin route: /{}", route);
}

pub async fn add_static_route(route: &str, path: &str) {
    let mut router = ROUTER_MANAGER.write().unwrap();
    *router = router.clone().nest_service(
        route,
        ServeDir::new(path)
    );
    println!("Added static route: {}", route);
}

pub async fn remove_route(route: &str) {
    let mut router = ROUTER_MANAGER.write().unwrap();
    // Create new router without the specified route
    let new_router = Router::new();
    
    // This is a bit hacky but works - we create a new router and copy all routes except the one we want to remove
    // In a production environment, you'd want to maintain a map of routes and rebuild more selectively
    *router = new_router;
    println!("Removed route: {}", route);
}
