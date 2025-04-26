use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::fs;
use axum::{Router, routing::{any, get}};
use axum::response::Response;
use axum::body::Body;
use axum::http::StatusCode;
use tower_http::services::ServeDir;

use engine_core::plugin_registry::PluginRegistry;
use engine_core::handlers::dispatch_plugin_api;

/// Global router manager for handling dynamic routes
static ROUTER_MANAGER: Lazy<Arc<RwLock<Router>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Router::new()))
});

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
    /// Maps {route} to static files in {path}.
    pub async fn add_plugin_route(route: &str, path: &str) {
        let mut router = ROUTER_MANAGER.write().unwrap();
        *router = router.clone().nest_service(
            &format!("/{}", route),
            ServeDir::new(path)
        );
        println!("Added plugin route: /{}", route);
    }

    /// Adds a static file route at runtime.
    /// Maps {route} to files in {path}.
    pub async fn add_static_route(route: &str, path: &str) {
        let mut router = ROUTER_MANAGER.write().unwrap();
        *router = router.clone().nest_service(
            route,
            ServeDir::new(path)
        );
        println!("Added static route: {}", route);
    }
}
