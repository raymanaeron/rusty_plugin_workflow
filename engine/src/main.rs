use axum::{Router, routing::any};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use engine_core::{plugin_loader::load_plugin, plugin_registry::PluginRegistry};
use engine_core::handlers::dispatch_plugin_api;
use tower_http::services::ServeDir;
use plugin_core::PluginContext;
use std::ffi::CString;
use logger::{ LoggerLoader, LogLevel};
use logger::log_contracts::Logger;
use logger::log_config::TelemetryConfig;
use toml;
use tokio::fs::File; 
// use tokio::io::BufReader;
use tokio::io::AsyncReadExt; 
use std::sync::OnceLock;

use axum::{
    routing::get,
    http::StatusCode,
    response::Response,
    body::Body,
};
use std::fs;

static LOGGER: OnceLock<Arc<dyn Logger>> = OnceLock::new();

async fn load_logger() {
    // Read configuration from app_config.toml
    let mut config_file = File::open("app_config.toml")
        .await
        .expect("Unable to open app_config.toml");
    let mut contents = String::new();
    config_file
        .read_to_string(&mut contents)
        .await
        .expect("Failed to read app_config.toml");

    // Parse the TOML configuration into TelemetryConfig
    let telemetry_config: TelemetryConfig =
        toml::from_str(&contents).expect("Error parsing app_config.toml");

    // Load the logger using the logging configuration
    let logger = LoggerLoader::load(&telemetry_config.logging);

    // Store the logger in the global LOGGER variable
    LOGGER.set(logger).expect("Logger already initialized");
}

fn logger() -> &'static Arc<dyn Logger> {
    LOGGER.get().expect("Logger is not initialized")
}

async fn fallback_handler() -> Response {
    match fs::read_to_string("webapp/index.html") {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Body::from(contents))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Failed to load fallback page"))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() {
    // Load the logger
    load_logger().await;

    logger().log(LogLevel::Info, "Creating plugin registry");

    // Create a plugin registry
    let registry = Arc::new(PluginRegistry::new());

    logger().log(LogLevel::Info, "Loading the wifi plugin");

    // Load the plugin
    let (plugin, _lib) = load_plugin("plugin_wifi.dll")
        .expect("Failed to load plugin");

    logger().log(LogLevel::Info, "Running the wifi plugin with a parameter");
    // Run plugin with a parameter
    let config = CString::new("scan=true").unwrap();
    let ctx = PluginContext {
        config: config.as_ptr(),
    };
    (plugin.run)(&ctx);

    logger().log(LogLevel::Info, "Registering wifi plugin");

    // Register plugin
    registry.register(plugin);

    // Build base router without state
    let mut app = Router::new();

    // Mount each plugin's routes
    for plugin in registry.all() {
        let api_path = format!("/{}/api/*path", plugin.name);
        println!("Api Path : {}", api_path);
        
        let web_path = format!("/{}/web", plugin.name);
        println!("Web Path : {}", web_path);

        let plugin_state = (registry.clone(), plugin.name.clone());

        app = app
            .route(&api_path, any(dispatch_plugin_api).with_state(plugin_state))
            .nest_service(&web_path, ServeDir::new(&plugin.static_path));

    }

    // Mount static shell assets from /webapp (for /app.js, /styles.css, etc.)
    app = app.nest_service("/", ServeDir::new("webapp"));

    // Fallback: for any unknown route like /wifi/web, return index.html
    app = app.fallback(get(fallback_handler));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening at http://{}", addr);

    // Use make_service_with_connect_info to bind the stateful router to axum::serve
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
    .await
    .unwrap();
}
