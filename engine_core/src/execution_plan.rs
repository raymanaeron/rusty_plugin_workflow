use crate::plugin_metadata::PluginMetadata;
use serde::Deserialize;
use std::{fs, path::Path, error::Error};

/// Root structure representing the dynamic plugin execution plan loaded from TOML.
#[derive(Debug, Deserialize)]
pub struct PluginExecutionPlan {
    /// List of plugin metadata entries to be dynamically loaded and executed.
    pub plugins: Vec<PluginMetadata>,
}

/// Loads and validates the plugin execution plan from a TOML file.
pub struct ExecutionPlanLoader;

impl ExecutionPlanLoader {
    /// Loads and parses the execution plan from a given path.
    /// Returns a fully validated and deserialized execution plan.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<PluginExecutionPlan, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let plan: PluginExecutionPlan = toml::from_str(&content)?;

        for (idx, plugin) in plan.plugins.iter().enumerate() {
            Self::validate_plugin(plugin, idx)?;
        }

        Ok(plan)
    }

    /// Validates all required fields for each plugin.
    fn validate_plugin(plugin: &PluginMetadata, index: usize) -> Result<(), Box<dyn Error>> {
        if plugin.full_name.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'full_name'", index).into());
        }
        if plugin.plugin_route.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'plugin_route'", index).into());
        }
        if plugin.version.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'version'", index).into());
        }
        if plugin.plugin_location_type != "local" && plugin.plugin_location_type != "s3" {
            return Err(format!(
                "Plugin at index {} has invalid 'plugin_location_type': must be 'local' or 's3'",
                index
            )
            .into());
        }
        if plugin.plugin_location_path.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'plugin_location_path'", index).into());
        }
        if plugin.team_name.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'team_name'", index).into());
        }
        if plugin.engineering_contact_email.trim().is_empty() {
            return Err(format!(
                "Plugin at index {} is missing 'engineering_contact_email'",
                index
            )
            .into());
        }
        if plugin.operation_contact_email.trim().is_empty() {
            return Err(format!(
                "Plugin at index {} is missing 'operation_contact_email'",
                index
            )
            .into());
        }

        Ok(())
    }
}
