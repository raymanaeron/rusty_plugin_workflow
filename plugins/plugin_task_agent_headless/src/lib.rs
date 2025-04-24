extern crate plugin_core;

use plugin_core::*;
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;
use plugin_core::ws_utils;

use std::ffi::{ CString, CStr };
use std::os::raw::c_char;
use std::ptr;
use std::sync::{ Mutex, Arc };
use std::thread;
use std::time::Duration;

use once_cell::sync::{ Lazy, OnceCell };
use ws_server::ws_client::WsClient;

#[ctor::ctor]
fn on_load() {
    println!("[plugin_task_agent_headless] >>> LOADED");
}

static PROGRESS_STATE: once_cell::sync::Lazy<Arc<Mutex<String>>> = once_cell::sync::Lazy::new(||
    Arc::new(Mutex::new("Waiting for job...".to_string()))
);

/// Topic for receiving status change messages.
pub static STATUS_CHANGED: &str = "StatusMessageChanged";

/// WebSocket client for the plugin.
pub static PLUGIN_WS_CLIENT: OnceCell<Arc<Mutex<WsClient>>> = OnceCell::new();

/// Shared Tokio runtime for the plugin.
pub static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
});

// WebSocket Client Initialization
/// Creates and initializes the WebSocket client for the plugin.
pub async fn create_ws_plugin_client() {
    println!("Creating ws client for the plugin");
    let url = "ws://127.0.0.1:8081/ws";

    // Connect to the WebSocket server.
    let client = WsClient::connect("plugin_task_agent", url)
        .await
        .expect("Failed to connect WsClient");

    // Store the WebSocket client in the static variable.
    if PLUGIN_WS_CLIENT.set(Arc::new(Mutex::new(client))).is_err() {
        eprintln!("Failed to set PLUGIN_WS_CLIENT: already initialized");
        return;
    }

    println!("ws client for the plugin_task_agent created");

    // Subscribe to the STATUS_CHANGED topic and set up a message handler.
    if let Some(client_arc) = PLUGIN_WS_CLIENT.get() {
        let mut client = client_arc.lock().unwrap();
        client
            .subscribe("plugin_task_agent", STATUS_CHANGED, "")
            .await;
        println!("Plugin, subscribed to STATUS_CHANGED");

        client.on_message(STATUS_CHANGED, |msg| {
            println!("[plugin_task_agent_headless] => STATUS_CHANGED: {}", msg);
        });
    }
}

extern "C" fn run(ctx: *const PluginContext) {
    println!("[plugin_task_agent_headless] - run");
    println!("[plugin_task_agent_headless] FINGERPRINT: run = {:p}", run as *const ());
    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    // Use shared runtime instead of creating new one
    RUNTIME.block_on(async {
        create_ws_plugin_client().await;
    });
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("taskagent/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 1] = [HttpMethod::Post];
    let slice = static_resource("jobs", &METHODS);
    unsafe {
        *out_len = slice.len();
    }
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

        match request.method {
            HttpMethod::Post if path == "jobs" => {
                // This allows us to kick off the run_workflow function
                // by invoking an HTTP POST on api/taskgent/jobs
                // of course we could also call run_worfklow directly
                // from the engine (where the plugin is loaded)
                // But this additional entry point is useful
                // if we wanted to run the workflow from a different plugin
                // or from a JS inside a webview of another plugin
                run_workflow(req)
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

extern "C" fn run_workflow(_req: *const ApiRequest) -> *mut ApiResponse {
    println!("[plugin_task_agent_headless] - run_workflow");

    thread::spawn(|| {
        let steps = vec![
            "Step 1: Initializing..",
            "Step 2: Processing..",
            "Step 3: Finalizing..",
            "Step 4: Completed",
        ];

        for step in steps {
            {
                let mut lock = PROGRESS_STATE.lock().unwrap();
                *lock = step.to_string();
            }

            // Try to publish with reconnection attempts
            if let Some(client_arc) = PLUGIN_WS_CLIENT.get() {
                let client_arc = client_arc.clone();
                let step = step.to_string();
                let timestamp = chrono::Utc::now().to_rfc3339();

                // Use shared runtime
                RUNTIME.block_on(async {
                    let mut retries = 3;
                    while retries > 0 {
                        if let Ok(mut client) = client_arc.lock() {
                            match client.publish("plugin_task_agent", STATUS_CHANGED, &step, &timestamp).await {
                                Ok(_) => {
                                    println!("[plugin_task_agent_headless] Successfully published status update");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("[plugin_task_agent_headless] Failed to publish status: {}", e);
                                    retries -= 1;
                                    if retries > 0 {
                                        tokio::time::sleep(Duration::from_millis(500)).await;
                                        // Try to reconnect
                                        if let Ok(new_client) = WsClient::connect("plugin_task_agent", "ws://127.0.0.1:8081/ws").await {
                                            *client = new_client;
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }

            thread::sleep(Duration::from_secs(2));
        }
    });

    json_response(202, r#"{ "message": "Job started" }"#)
}

extern "C" fn on_progress() -> *mut ApiResponse {
    let current = PROGRESS_STATE.lock().unwrap().clone();
    println!("[plugin_task_agent_headless] on_progress = {}", current);
    let msg = format!(r#"{{ "status": "{}" }}"#, current);
    json_response(200, &msg)
}

extern "C" fn on_complete() -> *mut ApiResponse {
    let current = PROGRESS_STATE.lock().unwrap().clone();
    if current == "Job completed" {
        json_response(200, r#"{ "message": "Job finished" }"#)
    } else {
        json_response(204, r#"{ "message": "Still running" }"#)
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

declare_plugin!(
    "plugin_task_agent_headless",
    "taskagent",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup,
    run_workflow,
    on_progress,
    on_complete
);
