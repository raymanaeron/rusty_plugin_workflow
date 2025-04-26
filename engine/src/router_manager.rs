use std::sync::Arc;
use axum::{Router, routing::{any, get}};
use tower_http::services::ServeDir;
use engine_core::plugin_registry::PluginRegistry;
use engine_core::handlers::dispatch_plugin_api;

pub struct RouterManager {
    registry: Arc<PluginRegistry>,
}

impl RouterManager {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
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
        app = app.fallback(get(crate::fallback_handler));

        app
    }
}
