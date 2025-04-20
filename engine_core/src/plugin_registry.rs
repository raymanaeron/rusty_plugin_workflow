use std::collections::HashMap;
use std::sync::RwLock;

use crate::plugin_binding::PluginBinding;

/// Central registry holding all plugins loaded into the engine.
/// Used by the Axum router to dynamically dispatch REST and static routes.
pub struct PluginRegistry {
    /// Maps internal plugin name (e.g., "plugin_wifi") to PluginBinding
    name_map: RwLock<HashMap<String, PluginBinding>>,

    /// Maps user-facing plugin route (e.g., "wifi") to PluginBinding
    route_map: RwLock<HashMap<String, PluginBinding>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            name_map: RwLock::new(HashMap::new()),
            route_map: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a plugin into both the name and route maps.
    pub fn register(&self, plugin: PluginBinding) {
        let mut name_map = self.name_map.write().unwrap();
        let mut route_map = self.route_map.write().unwrap();

        name_map.insert(plugin.name.clone(), plugin.clone());
        route_map.insert(plugin.plugin_route.clone(), plugin);
    }

    /// Gets a plugin by internal name.
    pub fn get(&self, name: &str) -> Option<PluginBinding> {
        let map = self.name_map.read().unwrap();
        map.get(name).cloned()
    }

    /// Gets a plugin by route (e.g., "wifi").
    pub fn get_by_route(&self, route: &str) -> Option<PluginBinding> {
        let map = self.route_map.read().unwrap();
        map.get(route).cloned()
    }

    /// Returns all registered plugins.
    pub fn all(&self) -> Vec<PluginBinding> {
        let map = self.name_map.read().unwrap();
        map.values().cloned().collect()
    }
}
