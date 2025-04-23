// Imports
use std::{ net::SocketAddr, sync::{ Arc, Mutex } };
use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;

use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

use once_cell::sync::{ Lazy, OnceCell };

use engine_core::{ plugin_loader::load_plugin, plugin_registry::PluginRegistry };
use engine_core::handlers::dispatch_plugin_api;
use engine_core::execution_plan_updater::{ ExecutionPlanUpdater, PlanLoadSource };
use engine_core::execution_plan::ExecutionPlanLoader;
use engine_core::plugin_utils;
use engine_core::plugin_metadata::PluginMetadata;
use engine_core::plugin_utils::prepare_plugin_binary;

use plugin_core::PluginContext;

use ws_server::{ handle_socket, Subscribers };
use ws_server::ws_client::WsClient;

use logger::{ LoggerLoader, LogLevel };

// Static Variables
pub static WS_SUBSCRIBERS: Lazy<Subscribers> = Lazy::new(|| {
    std::sync::Arc::new(
        std::sync::Mutex::new(HashMap::<String, Vec<UnboundedSender<String>>>::new())
    )
});

pub static STATUS_RECEIVED: &str = "StatusMessageReceived";

pub static ENGINE_WS_CLIENT: OnceCell<Arc<Mutex<WsClient>>> = OnceCell::new();

// WebSocket Client Initialization
/// Creates and initializes the WebSocket client for the engine.
pub async fn create_ws_engine_client() {
    println!("Creating ws client for the engine");
    let url = "ws://127.0.0.1:8081/ws";
    let client = WsClient::connect("engine", url).await.expect("Failed to connect WsClient");
    if ENGINE_WS_CLIENT.set(Arc::new(Mutex::new(client))).is_err() {
        eprintln!("Failed to set ENGINE_WS_CLIENT: already initialized");
        return;
    }

    println!("ws client for the engine created");

    if let Some(client_arc) = ENGINE_WS_CLIENT.get() {
        let mut client = client_arc.lock().unwrap();
        client.subscribe(STATUS_RECEIVED).await;
        println!("Engine, subscribed to STATUS_RECEIVED");
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
    let status_plugin = status_plugin.clone();
    let logger = LoggerLoader::get_logger();

    tokio::spawn(async move {
        use std::time::Duration;
        use tokio::time::sleep;
        use plugin_core::{ HttpMethod, ApiRequest };

        loop {
            if let Some(on_progress_fn) = task_agent.on_progress {
                let progress_resp_ptr = on_progress_fn();
                if !progress_resp_ptr.is_null() {
                    let progress_resp = unsafe { Box::from_raw(progress_resp_ptr) };

                    let status_json = unsafe {
                        std::slice::from_raw_parts(progress_resp.body_ptr, progress_resp.body_len)
                    };

                    let status_text = String::from_utf8_lossy(status_json);

                    logger.log(LogLevel::Debug, &format!("[engine] Progress: {}", status_text));

                    let path_cstr = std::ffi::CString::new("statusmessage").unwrap();

                    let req = ApiRequest {
                        method: HttpMethod::Post,
                        path: path_cstr.as_ptr(),
                        headers: std::ptr::null(),
                        header_count: 0,
                        body_ptr: progress_resp.body_ptr,
                        body_len: progress_resp.body_len,
                        content_type: progress_resp.content_type,
                        query: std::ptr::null(),
                    };

                    (status_plugin.handle_request)(&req);
                }
            }

            if let Some(on_complete_fn) = task_agent.on_complete {
                let done_ptr = on_complete_fn();
                if !done_ptr.is_null() {
                    let done_resp = unsafe { Box::from_raw(done_ptr) };
                    if done_resp.status == 200 {
                        logger.log(LogLevel::Info, "[engine] Job complete, stopping polling");
                        break;
                    }
                    (task_agent.cleanup)(Box::into_raw(done_resp));
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });

    let wifi_plugin = wifi_plugin.clone();
    let task_agent = task_agent_headless_plugin.clone();
    tokio::spawn(async move {
        use std::time::Duration;
        use tokio::time::sleep;
        use plugin_core::{ HttpMethod, ApiRequest };

        loop {
            if let Some(wifi_complete_fn) = wifi_plugin.on_complete {
                let resp_ptr = wifi_complete_fn();
                if !resp_ptr.is_null() {
                    let resp = unsafe { Box::from_raw(resp_ptr) };
                    if resp.status == 200 {
                        LoggerLoader::get_logger().log(LogLevel::Info, "WiFi is connected");

                        if let Some(run_workflow_fn) = task_agent.run_workflow {
                            LoggerLoader::get_logger().log(
                                LogLevel::Info,
                                "Triggering task_agent_headless run_workflow"
                            );

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

                        break;
                    }

                    (wifi_plugin.cleanup)(Box::into_raw(resp));
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });

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

    let mut app = Router::new();

    use axum::routing::any;
    use axum::Router;

    let plugin_api_router = Router::new().route(
        "/:plugin/:resource",
        any(dispatch_plugin_api).with_state(registry.clone())
    );

    app = app.nest("/api", plugin_api_router);

    use tower_http::services::ServeDir;
    for plugin in registry.all() {
        let web_path = format!("/{}/web", plugin.plugin_route);
        println!("-> registered plugin name: {}", plugin.name);
        println!("Web Path : {}", web_path);
        app = app.nest_service(&web_path, ServeDir::new(&plugin.static_path));
    }

    app = app.nest_service("/", ServeDir::new("webapp"));

    use axum::{ routing::get, response::Response, http::StatusCode, body::Body };
    use std::fs;

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

    app = app.fallback(get(fallback_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
