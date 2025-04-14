use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use plugin_loader::{load_plugin, LoadedPlugin};
use plugin_api::{PluginContext};

use axum::Json;
use serde::Serialize;
use plugin_api::NetworkInfo;
use std::ffi::CStr;
use libloading::Symbol;

mod plugin_loader;

#[derive(Clone)]
struct AppState {
    plugin: Arc<LoadedPlugin>,
}

#[tokio::main]
async fn main() {
    let plugin = load_plugin("plugin_wifi.dll").expect("Failed to load plugin");

    // Create context with static config
    let config_str = std::ffi::CString::new("scan=true").unwrap();
    let context = PluginContext {
        config: config_str.as_ptr(),
    };

    // Run the plugin
    (plugin.api.run)(&context);

    // Wrap plugin in AppState
    let state = Arc::new(AppState {
        plugin: Arc::new(plugin),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/wifi/scan", get(scan_wifi))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Starting Axum server on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html("<h1>Hello from Engine Web Server</h1>")
}

#[derive(Serialize)]
struct NetworkInfoJson {
    ssid: String,
    bssid: String,
    signal: i32,
    channel: i32,
    security: String,
    frequency: f32,
}

async fn scan_wifi(State(state): State<Arc<AppState>>) -> Json<Vec<NetworkInfoJson>> {
    unsafe {
        let scan_fn: Symbol<unsafe extern "C" fn(*mut usize) -> *mut NetworkInfo> =
            match state.plugin._lib.get(b"scan\0") {
                Ok(f) => f,
                Err(_) => return Json(vec![]),
            };

        let mut count: usize = 0;
        let result_ptr = scan_fn(&mut count as *mut usize);

        if result_ptr.is_null() || count == 0 {
            return Json(vec![]);
        }

        let results: &[NetworkInfo] = std::slice::from_raw_parts(result_ptr, count);

        let json_results: Vec<NetworkInfoJson> = results
            .iter()
            .map(|net| {
                NetworkInfoJson {
                    ssid: CStr::from_ptr(net.ssid).to_string_lossy().into_owned(),
                    bssid: CStr::from_ptr(net.bssid).to_string_lossy().into_owned(),
                    signal: net.signal,
                    channel: net.channel,
                    security: CStr::from_ptr(net.security).to_string_lossy().into_owned(),
                    frequency: net.frequency,
                }
            })
            .collect();

        Json(json_results)
    }
}
