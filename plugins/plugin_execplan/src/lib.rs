extern crate plugin_core;

use plugin_core::{
    ApiRequest, ApiResponse, HttpMethod, PluginContext, Resource,
    declare_plugin,
    error_response,
    response_utils::{json_response, method_not_allowed_response},
    resource_utils::static_resource,
    cleanup_response,
};
use std::sync::{Arc, Mutex};
use libws::ws_client::WsClient;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;
use std::os::raw::c_char;
use std::ffi::{CString, CStr};
use std::ptr;
use serde::{Serialize, Deserialize};

// Shared Runtime for async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// Shared WebSocket client
static mut PLUGIN_WS_CLIENT: Option<Arc<Mutex<WsClient>>> = None;

// Define your data structure
#[derive(Serialize, Deserialize, Clone, Default)]
struct BluePrint {
    // Add your fields here
    field1: String,
    field2: bool,
}

// Shared state
static STATE: Lazy<Mutex<BluePrint>> = Lazy::new(|| {
    Mutex::new(BluePrint::default())
});

#[ctor::ctor]
fn on_load() {
    println!("[plugin_execplan] >>> LOADED");
}

pub async fn create_ws_plugin_client() {
    if let Ok(client) = WsClient::connect("plugin_execplan", "ws://127.0.0.1:8081/ws").await {
        let client = Arc::new(Mutex::new(client));
        /*
        if let Ok(mut ws_client) = client.lock() {
            //ws_client.subscribe("plugin_execplan", "BluePrinttUpdated", "").await;
            //println!("[plugin_execplan] Subscribed to BluePrintUpdated");
        }
        */
        unsafe {
            PLUGIN_WS_CLIENT = Some(client);
        }
    }
}

extern "C" fn run(_ctx: *const PluginContext) {
    println!("[plugin_execplan] - run");
    RUNTIME.block_on(async {
        create_ws_plugin_client().await;
    });
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("execution/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 4] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
    ];
    let slice = static_resource("blueprint", &METHODS);
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
            HttpMethod::Get if path == "blueprint" => {
                let current = STATE.lock().unwrap().clone();
                let json = serde_json::to_string(&current).unwrap();
                json_response(200, &json)
            }

            HttpMethod::Post if path == "blueprint" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(data) = serde_json::from_slice::<BluePrint>(body) {
                    let mut state = STATE.lock().unwrap();
                    *state = data;
                    json_response(201, r#"{"message": "Resource created"}"#)
                } else {
                    error_response(400, "Invalid data")
                }
            }

            HttpMethod::Put if path == "blueprint" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(data) = serde_json::from_slice::<BluePrint>(body) {
                    let mut state = STATE.lock().unwrap();
                    *state = data;
                    json_response(200, r#"{"message": "Resource updated"}"#)
                } else {
                    error_response(400, "Invalid data")
                }
            }

            HttpMethod::Delete if path == "blueprint" => {
                let mut state = STATE.lock().unwrap();
                *state = BluePrint::default();
                json_response(200, r#"{"message": "Resource reset to defaults"}"#)
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

declare_plugin!(
    "plugin_execplan",
    "execution",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);
