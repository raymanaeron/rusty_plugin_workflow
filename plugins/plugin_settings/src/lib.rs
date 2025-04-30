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

// DeviceSettings is the top-level struct holding all plugin settings.
// It contains three sections: general, echo, and automation.
#[derive(Serialize, Deserialize, Clone, Default)]
struct DeviceSettings {
    general: GeneralSettings,
    echo: EchoSettings,
    automation: AutomationSettings,
}

// GeneralSettings holds general device configuration such as name, language, region, etc.
#[derive(Serialize, Deserialize, Clone)]
struct GeneralSettings {
    #[serde(rename = "deviceName")]
    device_name: String,      // Device name as shown to the user
    language: String,         // Language code (e.g., "en-US")
    region: String,           // Region code (e.g., "us")
    #[serde(rename = "timeZone")]
    time_zone: String,        // Time zone string (e.g., "GMT-5")
    #[serde(rename = "autoUpdate")]
    auto_update: bool,        // Whether automatic updates are enabled
    #[serde(rename = "amazonEmail")]
    amazon_email: String,     // Amazon account email
    #[serde(rename = "shareMetrics")]
    share_metrics: bool,      // Whether to share device metrics
}

// Provides default values for GeneralSettings.
impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            device_name: "My Echo".to_string(),
            language: "en-US".to_string(),
            region: "us".to_string(),
            time_zone: "GMT-5".to_string(),
            auto_update: true,
            amazon_email: "".to_string(),
            share_metrics: true,
        }
    }
}

// EchoSettings holds configuration specific to Echo device features.
#[derive(Serialize, Deserialize, Clone)]
struct EchoSettings {
    #[serde(rename = "wakeWord")]
    wake_word: String,            // Wake word for the device (e.g., "Alexa")
    #[serde(rename = "micEnabled")]
    mic_enabled: bool,            // Whether the microphone is enabled
    #[serde(rename = "dropInCalling")]
    drop_in_calling: bool,        // Whether Drop In/Calling is enabled
    #[serde(rename = "displaySettings")]
    display_settings: String,     // Display setting (e.g., "brightness")
}

// Provides default values for EchoSettings.
impl Default for EchoSettings {
    fn default() -> Self {
        Self {
            wake_word: "Alexa".to_string(),
            mic_enabled: true,
            drop_in_calling: false,
            display_settings: "brightness".to_string(),
        }
    }
}

// AutomationSettings holds automation-related configuration.
#[derive(Serialize, Deserialize, Clone)]
struct AutomationSettings {
    #[serde(rename = "frustrationFreeAutomation")]
    frustration_free_automation: bool, // Whether Frustration Free Automation is enabled
}

// Provides default values for AutomationSettings.
impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            frustration_free_automation: true,
        }
    }
}

// Shared state
static STATE: Lazy<Mutex<DeviceSettings>> = Lazy::new(|| {
    Mutex::new(DeviceSettings {
        general: GeneralSettings::default(),
        echo: EchoSettings::default(),
        automation: AutomationSettings::default(),
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