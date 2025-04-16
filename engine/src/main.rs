use axum::{Router, routing::any};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use engine_core::{plugin_loader::load_plugin, plugin_registry::PluginRegistry};
use engine_core::handlers::dispatch_plugin_api;
use tower_http::services::ServeDir;
use plugin_core::PluginContext;
use std::ffi::CString;

use axum::{
    routing::get,
    http::{StatusCode, HeaderValue},
    response::{IntoResponse, Response},
    body::Body,
};
use std::fs;


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
    let registry = Arc::new(PluginRegistry::new());

    // Load the plugin
    let (plugin, _lib) = load_plugin("plugin_wifi.dll")
        .expect("Failed to load plugin");

    // Run plugin with config
    let config = CString::new("scan=true").unwrap();
    let ctx = PluginContext {
        config: config.as_ptr(),
    };
    (plugin.run)(&ctx);

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
