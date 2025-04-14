use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use tokio::net::TcpListener;

mod plugin_loader; 

use plugin_loader::load_plugin;
use plugin_api::{PluginApi, PluginContext};

struct AppState {
    plugin: Arc<PluginApi>,
}

#[tokio::main]
async fn main() {

    let plugin = load_plugin("plugin_wifi.dll").expect("Failed to load plugin");

    //let plugin = load_plugin("plugins/plugin_wifi/target/debug/plugin_wifi.dll")
    // .expect("Failed to load plugin");


    // Create context with static config
    let config_str = std::ffi::CString::new("scan=true").unwrap();
    let context = PluginContext {
        config: config_str.as_ptr(),
    };

    (plugin.api.run)(&context);

    let state = AppState {
        plugin: plugin.api.clone(),
    };

    let app = Router::new()
        .route("/api/wifi/scan", get(scan_wifi))
        .route("/", get(index))
        .with_state(Arc::new(state));

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html("<h1>Hello from Engine Web Server</h1>")
}

async fn scan_wifi(State(state): State<Arc<AppState>>) -> Html<String> {
    let name_cstr = unsafe { std::ffi::CStr::from_ptr((state.plugin.name)()) };
    let plugin_name = name_cstr.to_string_lossy();

    let result = format!("WiFi scan executed by plugin: {}", plugin_name);
    Html(result)
}
