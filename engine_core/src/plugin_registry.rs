use std::collections::HashMap;
use std::sync::RwLock;

use crate::plugin_binding::PluginBinding;

/// Central registry holding all plugins loaded into the engine.
/// Used by the Axum router to dynamically dispatch REST and static routes.
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, PluginBinding>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, plugin: PluginBinding) {
        let mut map = self.plugins.write().unwrap();
        map.insert(plugin.name.clone(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<PluginBinding> {
        let map = self.plugins.read().unwrap();
        map.get(name).cloned()
    }

    pub fn all(&self) -> Vec<PluginBinding> {
        let map = self.plugins.read().unwrap();
        map.values().cloned().collect()
    }
}
