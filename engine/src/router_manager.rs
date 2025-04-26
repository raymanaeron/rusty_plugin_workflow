//! Router Manager module for handling dynamic route management in the plugin system.
//! Provides functionality for managing HTTP routes and serving static files.

// Standard library imports
use std::fs;
use std::sync::{Arc, RwLock};

// Third-party imports
use axum::{
    body::Body,
    http::StatusCode,
    response::Response,
    routing::{any, get},
    Router,
};
use once_cell::sync::Lazy;
use tower_http::services::ServeDir;

// Local imports
use engine_core::{
    handlers::dispatch_plugin_api,
    plugin_registry::PluginRegistry,
};

/// Global router manager for handling dynamic routes.
/// Uses a lazy-initialized RwLock to allow runtime modifications.
static ROUTER_MANAGER: Lazy<Arc<RwLock<Router>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Router::new()))
});

/// RouterManager handles the creation and management of HTTP routes for the plugin system.
/// It maintains routes for API endpoints, static files, and plugin-specific web content.
#[allow(dead_code)]
pub struct RouterManager {
    registry: Arc<PluginRegistry>,
}

#[allow(dead_code)]
impl RouterManager {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }

    /// Get a reference to the global router manager
    pub fn get_manager() -> &'static Lazy<Arc<RwLock<Router>>> {
        &ROUTER_MANAGER
    }

    /// Default fallback handler that serves index.html
    pub async fn fallback_handler() -> Response {
        match fs::read_to_string("webapp/index.html") {
            Ok(content) =>
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Body::from(content))
                    .unwrap(),
            Err(_) =>
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("index.html not found"))
                    .unwrap(),
        }
    }

    /// Builds the initial router configuration with all routes:
    /// - API routes for plugin endpoints
    /// - Plugin-specific web routes
    /// - Static webapp routes
    /// - Fallback handler for SPA support
    pub fn build_routes(&self) -> Router {
        let mut app = Router::new();

        // API routes
        let plugin_api_router = Router::new().route(
            "/:plugin/:resource",
            any(dispatch_plugin_api).with_state(self.registry.clone())
        );
        app = app.nest("/api", plugin_api_router);

        // Plugin web routes
        for plugin in self.registry.all() {
            let web_path = format!("/{}/web", plugin.plugin_route);
            app = app.nest_service(&web_path, ServeDir::new(&plugin.static_path));
        }

        // Static webapp route
        app = app.nest_service("/", ServeDir::new("webapp"));
        // Update fallback handler reference
        app = app.fallback(get(Self::fallback_handler));

        app
    }

    /// Adds a new plugin route at runtime.
    /// 
    /// # Arguments
    /// * `route` - The URL path segment for the plugin
    /// * `path` - The filesystem path to the plugin's static files
    pub async fn add_plugin_route(route: &str, path: &str) {
        let mut router = ROUTER_MANAGER.write().unwrap();
        *router = router.clone().nest_service(
            &format!("/{}", route),
            ServeDir::new(path)
        );
        println!("Added plugin route: /{}", route);
    }

    /// Adds a static file route at runtime.
    /// 
    /// # Arguments
    /// * `route` - The URL path to serve the files under
    /// * `path` - The filesystem path to the static files
    pub async fn add_static_route(route: &str, path: &str) {
        let mut router = ROUTER_MANAGER.write().unwrap();
        *router = router.clone().nest_service(
            route,
            ServeDir::new(path)
        );
        println!("Added static route: {}", route);
    }
}
