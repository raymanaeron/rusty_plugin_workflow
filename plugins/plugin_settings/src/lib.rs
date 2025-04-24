use plugin_core::{
    ApiRequest, ApiResponse, HttpMethod, PluginContext, Resource,
    declare_plugin,
    error_response,
    response_utils::{json_response, method_not_allowed_response},
    resource_utils::static_resource,
    cleanup_response,
};
use std::sync::{Arc, Mutex};
use ws_server::ws_client::WsClient;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;
use std::os::raw::c_char;
use std::ffi::CString;
use std::ffi::CStr;
use std::ptr;
use serde::Serialize;
use serde::Deserialize;

// Shared Runtime
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// Shared WebSocket client
static mut PLUGIN_WS_CLIENT: Option<Arc<Mutex<WsClient>>> = None;

// Device Settings structure
#[derive(Serialize, Deserialize, Clone, Default)]
struct DeviceSettings {
    timezone: String,
    language: String,
    metrics_enabled: bool,
    copy_settings: bool,
    theme: String,
}

// Shared state
static STATE: Lazy<Mutex<DeviceSettings>> = Lazy::new(|| {
    Mutex::new(DeviceSettings {
        timezone: "UTC".to_string(),
        language: "en-US".to_string(),
        metrics_enabled: false,
        copy_settings: false,
        theme: "light".to_string(),
    })
});

#[ctor::ctor]
fn on_load() {
    println!("[plugin_settings] >>> LOADED");
}

// Create WebSocket client
pub async fn create_ws_plugin_client() {
    if let Ok(client) = WsClient::connect("plugin_settings", "ws://127.0.0.1:8081/ws").await {
        let client = Arc::new(Mutex::new(client));
        
        if let Ok(mut ws_client) = client.lock() {
            ws_client.subscribe("plugin_settings", "SettingUpdateCompleted", "").await;
            println!("[plugin_settings] Subscribed to SettingUpdateCompleted");
        }
        
        unsafe {
            PLUGIN_WS_CLIENT = Some(client);
        }
    }
}

extern "C" fn run(_ctx: *const PluginContext) {
    println!("[plugin_settings] - run");
    RUNTIME.block_on(async {
        create_ws_plugin_client().await;
    });
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("settings/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 4] = [
        HttpMethod::Get, 
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
    ];
    let slice = static_resource("devicesettings", &METHODS);
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
            HttpMethod::Get if path == "devicesettings" => {
                let current = STATE.lock().unwrap().clone();
                let json = serde_json::to_string(&current).unwrap();
                json_response(200, &json)
            }

            HttpMethod::Post if path == "devicesettings" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(settings) = serde_json::from_slice::<DeviceSettings>(body) {
                    let mut state = STATE.lock().unwrap();
                    *state = settings;
                    json_response(201, r#"{"message": "Settings created"}"#)
                } else {
                    error_response(400, "Invalid settings data")
                }
            }

            HttpMethod::Put if path == "devicesettings" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                if let Ok(settings) = serde_json::from_slice::<DeviceSettings>(body) {
                    let mut state = STATE.lock().unwrap();
                    *state = settings;
                    json_response(200, r#"{"message": "Settings updated"}"#)
                } else {
                    error_response(400, "Invalid settings data")
                }
            }

            HttpMethod::Delete if path == "devicesettings" => {
                let mut state = STATE.lock().unwrap();
                *state = DeviceSettings::default();
                json_response(200, r#"{"message": "Settings reset to defaults"}"#)
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

declare_plugin!(
    "plugin_settings",
    "settings",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup
);