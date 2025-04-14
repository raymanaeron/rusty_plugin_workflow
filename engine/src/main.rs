use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use plugin_loader::{load_plugin, LoadedPlugin};
use plugin_api::{PluginContext};

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

async fn scan_wifi(State(state): State<Arc<AppState>>) -> Html<String> {
    let name_cstr = unsafe { std::ffi::CStr::from_ptr((state.plugin.api.name)()) };
    let plugin_name = name_cstr.to_string_lossy();

    let result = format!("WiFi scan executed by plugin: {}", plugin_name);
    Html(result)
}
