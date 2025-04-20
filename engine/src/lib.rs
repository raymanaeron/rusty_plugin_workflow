use std::{ net::SocketAddr, sync::Arc };
use tokio::net::TcpListener;
use engine_core::{ plugin_loader::load_plugin, plugin_registry::PluginRegistry };
use engine_core::handlers::dispatch_plugin_api;
use plugin_core::PluginContext;
use logger::{ LoggerLoader, LogLevel };
use std::ffi::CString;
use engine_core::execution_plan_updater::ExecutionPlanUpdater;
use engine_core::execution_plan::ExecutionPlanLoader;
use engine_core::plugin_utils;
use engine_core::plugin_metadata::PluginMetadata;
use engine_core::plugin_utils::prepare_plugin_binary;
use std::path::PathBuf;
use engine_core::execution_plan_updater::PlanLoadSource;

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

/// FFI-safe C entry point for Swift, Kotlin, C++, etc.
#[no_mangle]
pub extern "C" fn start_oobe_server() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });
}

/// /// Native async entry point for Rust-based apps (e.g. desktop, CLI)
pub async fn start_server_async() {
    // Load the logger
    LoggerLoader::init("app_config.toml").await;

    let logger = LoggerLoader::get_logger();
    logger.log(LogLevel::Info, "Logger initialized");
    logger.log(LogLevel::Info, "Creating plugin registry");

    // Create a plugin registry
    let registry = Arc::new(PluginRegistry::new());

    // We hold the libs in memory to avoid dropping them
    let mut plugin_libraries = Vec::new();

    // Load the terms plugin
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

    // Load the wifi plugin
    logger.log(LogLevel::Info, "Loading the wifi plugin");
    // let (wifi_plugin, _wifi_lib) = load_plugin("plugin_wifi.dll").expect("Failed to load plugin");
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
    registry.register(wifi_plugin);

    // Load the status plugin
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

    // clone it because we will need to reuse it
    registry.register(status_plugin.clone());

    // Load the task_agent_headless plugin
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

    // clone it because we will need to reuse it
    registry.register(task_agent_headless_plugin.clone());

    // Core plugins loaded
    logger.log(LogLevel::Info, "Core plugins loaded");

    // We are going to run a workflow on the task agent headless plugin
    // First subscribe to the on_prgress and on_complete callbacks
    let task_agent = task_agent_headless_plugin.clone();
    let status_plugin = status_plugin.clone();
    let logger = LoggerLoader::get_logger();

    tokio::spawn(async move {
        use std::time::Duration;
        use tokio::time::sleep;
        use plugin_core::{ HttpMethod, ApiRequest };

        loop {
            // Call on_progress (if available)
            if let Some(on_progress_fn) = task_agent.on_progress {
                let progress_resp_ptr = on_progress_fn();
                if !progress_resp_ptr.is_null() {
                    let progress_resp = unsafe { Box::from_raw(progress_resp_ptr) };

                    let status_json = unsafe {
                        std::slice::from_raw_parts(
                            progress_resp.body_ptr,
                            progress_resp.body_len
                        )
                    };
                    
                    let status_text = String::from_utf8_lossy(status_json);

                    logger.log(LogLevel::Debug, &format!("[engine] Progress: {}", status_text));

                    // Send it to plugin_status
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

                    // Do not double-free pointers, transfer ownership only once
                    // So do NOT call cleanup here
                }
            }

            // Check for completion
            if let Some(on_complete_fn) = task_agent.on_complete {
                let done_ptr = on_complete_fn();
                if !done_ptr.is_null() {
                    let done_resp = unsafe { Box::from_raw(done_ptr) };
                    if done_resp.status == 200 {
                        logger.log(LogLevel::Info, "[engine] Job complete, stopping polling");
                        break;
                    }
                    // Drop/cleanup here, since we allocated response
                    (task_agent.cleanup)(Box::into_raw(done_resp));
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });

    // Now we can trigger the workflow
    // This is a blocking call, so we need to run it in a separate thread
    if let Some(run_workflow_fn) = task_agent_headless_plugin.run_workflow {
        logger.log(LogLevel::Info, "Triggering task_agent_headless run_workflow");

        let json_bytes = r#"{"task": "background_job"}"#.as_bytes().to_vec();
        let json_len = json_bytes.len();
        let body_ptr = Box::into_raw(json_bytes.into_boxed_slice()) as *const u8;
        
        let request = plugin_core::ApiRequest {
            method: plugin_core::HttpMethod::Post,
            path: std::ptr::null(), // Or CString::new("job").unwrap().into_raw() if needed
            headers: std::ptr::null(),
            header_count: 0,
            body_ptr,
            body_len: json_len, 
            content_type: std::ptr::null(),
            query: std::ptr::null(),
        };

        // This will start the task
        let _ = run_workflow_fn(&request);
    }

    // Let's load the execution plan
    // Add discovered plugins to the registry
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

    // Build base router without state
    let mut app = Router::new();

    // 1. Shared plugin API route for all plugins
    // This matches paths like /terms/api/userterms or /wifi/api/network
    use axum::routing::any;
    use axum::Router;

    let plugin_api_router = Router::new().route(
        "/:plugin/:resource",
        any(dispatch_plugin_api).with_state(registry.clone())
    );

    // Mount it under /api to avoid conflicts
    app = app.nest("/api", plugin_api_router);

    // 2. Plugin static content like /terms/web/* or /wifi/web/*
    use tower_http::services::ServeDir;
    for plugin in registry.all() {
        let web_path = format!("/{}/web", plugin.plugin_route);
        println!("-> registered plugin name: {}", plugin.name);
        println!("Web Path : {}", web_path);
        app = app.nest_service(&web_path, ServeDir::new(&plugin.static_path));
    }

    // 3. Serve root UI assets (index.html, app.js, styles.css) from /webapp
    app = app.nest_service("/", ServeDir::new("webapp"));

    // 4. Fallback for unknown routes to index.html
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

    // Use make_service_with_connect_info to bind the stateful router to axum::serve
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
