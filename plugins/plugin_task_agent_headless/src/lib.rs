extern crate plugin_core;

use plugin_core::*;
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

#[ctor::ctor]
fn on_load() {
    println!("[plugin_task_agent_headless] >>> LOADED");
}

static PROGRESS_STATE: once_cell::sync::Lazy<Arc<Mutex<String>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new("Waiting for job...".to_string())));

extern "C" fn run(ctx: *const PluginContext) {
    println!("[plugin_task_agent_headless] - run");
    println!("[plugin_task_agent_headless] FINGERPRINT: run = {:p}", run as *const ());
    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config = CStr::from_ptr((*ctx).config);
        println!("Plugin running with config: {}", config.to_string_lossy());
    }
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("taskagent/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 1] = [HttpMethod::Post];
    let slice = static_resource("jobs", &METHODS);
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

extern "C" fn run_workflow(req: *const ApiRequest) -> *mut ApiResponse {
    println!("[plugin_task_agent_headless] - run_workflow input request: {:?}", req);
    let state = PROGRESS_STATE.clone();
    {
        let mut lock = state.lock().unwrap();
        *lock = "Starting background job...".to_string();
    }

    thread::spawn(move || {
        let mut state = state.lock().unwrap();
        *state = "Step 1: initializing...".to_string();
        drop(state);
        thread::sleep(Duration::from_secs(2));

        let mut state = PROGRESS_STATE.lock().unwrap();
        *state = "Step 2: processing...".to_string();
        drop(state);
        thread::sleep(Duration::from_secs(2));

        let mut state = PROGRESS_STATE.lock().unwrap();
        *state = "Step 3: finalizing...".to_string();
        drop(state);
        thread::sleep(Duration::from_secs(2));

        let mut state = PROGRESS_STATE.lock().unwrap();
        *state = "Job completed".to_string();
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

