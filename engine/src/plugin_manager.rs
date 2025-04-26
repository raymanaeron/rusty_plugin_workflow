//! Plugin Manager Module
//! 
//! This module provides functionality for loading and managing plugins at runtime.
//! It handles plugin lifecycle, registration, and resource management.

// Standard library imports
use std::sync::Arc;
use std::ffi::CString;

// Internal crate imports
use plugin_core::PluginContext;
use engine_core::{
    plugin_loader::load_plugin,
    plugin_registry::PluginRegistry,
    plugin_utils,
    PluginBinding,
};

// External crate imports
use libloading::Library;

/// Manages the lifecycle of plugins including loading, registration, and cleanup.
/// 
/// The `PluginManager` maintains a registry of loaded plugins and their associated
/// dynamic libraries to ensure proper resource management.
pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    pub(crate) plugin_libraries: Vec<Library>,
}

impl PluginManager {
    /// Creates a new instance of the plugin manager.
    /// 
    /// # Arguments
    /// * `registry` - A thread-safe reference to the plugin registry
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            registry,
            plugin_libraries: Vec::new(),
        }
    }

    /// Loads and initializes a plugin from a dynamic library.
    /// 
    /// # Arguments
    /// * `plugin_name` - Name of the plugin to load
    /// * `config` - Configuration string to pass to the plugin
    /// 
    /// # Returns
    /// * `Option<PluginBinding>` - The plugin binding if successfully loaded, None otherwise
    pub fn load_plugin(&mut self, plugin_name: &str, config: &str) -> Option<PluginBinding> {
        println!("Loading the {} plugin", plugin_name);

        let (plugin, lib) = match load_plugin(plugin_utils::resolve_plugin_filename(plugin_name)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to load {} plugin: {}", plugin_name, e);
                return None;
            }
        };

        println!("Running the {} plugin with a parameter", plugin_name);
        let plugin_config = CString::new(config).unwrap();
        let ctx = PluginContext {
            config: plugin_config.as_ptr(),
        };
        (plugin.run)(&ctx);

        println!("Registering {} plugin", plugin_name);
        self.plugin_libraries.push(lib);
        let binding = PluginBinding::from(plugin);
        self.registry.register(binding.clone());

        Some(binding)
    }

    /// Returns a mutable reference to the collection of loaded plugin libraries.
    /// 
    /// This method is primarily used for internal crate access to manage plugin cleanup.
    pub fn get_plugin_libraries(&mut self) -> &mut Vec<Library> {
        &mut self.plugin_libraries
    }
}
