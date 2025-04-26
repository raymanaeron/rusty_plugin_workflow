use std::sync::Arc;
use std::ffi::CString;
use plugin_core::PluginContext;
use engine_core::{plugin_loader::load_plugin, plugin_registry::PluginRegistry, PluginBinding};
use engine_core::plugin_utils;
use libloading::Library;
use logger::{Logger, LogLevel};

pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    pub(crate) plugin_libraries: Vec<Library>,  // Made public(crate)
    logger: Arc<dyn Logger>,
}

impl PluginManager {
    pub fn new(registry: Arc<PluginRegistry>, logger: Arc<dyn Logger>) -> Self {
        Self {
            registry,
            plugin_libraries: Vec::new(),
            logger,
        }
    }

    pub fn load_plugin(&mut self, plugin_name: &str, config: &str) -> Option<PluginBinding> {
        self.logger.log(LogLevel::Info, &format!("Loading the {} plugin", plugin_name));

        let (plugin, lib) = match load_plugin(plugin_utils::resolve_plugin_filename(plugin_name)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to load {} plugin: {}", plugin_name, e);
                return None;
            }
        };

        self.logger.log(LogLevel::Info, &format!("Running the {} plugin with a parameter", plugin_name));
        let plugin_config = CString::new(config).unwrap();
        let ctx = PluginContext {
            config: plugin_config.as_ptr(),
        };
        (plugin.run)(&ctx);

        self.logger.log(LogLevel::Info, &format!("Registering {} plugin", plugin_name));
        self.plugin_libraries.push(lib);
        let binding = PluginBinding::from(plugin);
        self.registry.register(binding.clone());

        Some(binding)
    }

    // Add getter for plugin_libraries if needed outside crate
    pub fn get_plugin_libraries(&mut self) -> &mut Vec<Library> {
        &mut self.plugin_libraries
    }
}
