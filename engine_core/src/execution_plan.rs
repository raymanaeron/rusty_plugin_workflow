use crate::plugin_metadata::PluginMetadata;
use serde::Deserialize;
use std::{fs, path::Path, error::Error};

#[derive(Debug, Deserialize)]
pub struct PluginExecutionPlan {
    pub general: GeneralConfig,
    pub plugins: Vec<PluginMetadata>,
    pub handoffs: Handoffs,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub product_family: String,
    pub execution_plan_version: String,
    pub update_from: String,
    pub update_path_root: String,
}

#[derive(Debug, Deserialize)]
pub struct Handoffs {
    pub handoff_events: Vec<String>,
}

pub struct ExecutionPlanLoader;

impl ExecutionPlanLoader {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<PluginExecutionPlan, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let plan: PluginExecutionPlan = toml::from_str(&content)?;

        // general section validation
        Self::validate_general(&plan.general)?;
        
        // plugins section validation
        for (idx, plugin) in plan.plugins.iter().enumerate() {
            Self::validate_plugin(plugin, idx)?;
        }

        // handoffs section validation
        Self::validate_handoffs(&plan.handoffs)?;

        Ok(plan)
    }

    fn validate_handoffs(handoffs: &Handoffs) -> Result<(), Box<dyn Error>> {
        for event in &handoffs.handoff_events {
            println!("Handoff event: {}", event);
        }
        Ok(())
    }

    fn validate_general(general: &GeneralConfig) -> Result<(), Box<dyn Error>> {
        if general.product_family.trim().is_empty() {
            return Err("Missing 'product_family' in [general] section".into());
        }
        if general.execution_plan_version.trim().is_empty() {
            return Err("Missing 'execution_plan_version' in [general] section".into());
        }
        if general.update_from != "s3" && general.update_from != "local" && general.update_from != "unc" {
            return Err("Invalid 'update_from' in [general] section: must be 's3', 'local', or 'unc'".into());
        }
        if general.update_path_root.trim().is_empty() {
            return Err("Missing 'update_path_root' in [general] section".into());
        }

        Ok(())
    }

    fn validate_plugin(plugin: &PluginMetadata, index: usize) -> Result<(), Box<dyn Error>> {
        if plugin.name.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'name'", index).into());
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
            ).into());
        }
        if plugin.plugin_base_path.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'plugin_base_path'", index).into());
        }
        if plugin.team_name.trim().is_empty() {
            return Err(format!("Plugin at index {} is missing 'team_name'", index).into());
        }
        if plugin.engineering_contact_email.trim().is_empty() {
            return Err(format!(
                "Plugin at index {} is missing 'engineering_contact_email'",
                index
            ).into());
        }
        if plugin.operation_contact_email.trim().is_empty() {
            return Err(format!(
                "Plugin at index {} is missing 'operation_contact_email'",
                index
            ).into());
        }
        // Validate optional run_after_event_name if present
        if let Some(event_name) = &plugin.run_after_event_name {
            if event_name.trim().is_empty() {
                return Err(format!(
                    "Plugin at index {} has empty 'run_after_event_name'",
                    index
                ).into());
            }
        }

        // Validate optional completed_event_name if present
        if let Some(event_name) = &plugin.completed_event_name {
            if event_name.trim().is_empty() {
                return Err(format!(
                    "Plugin at index {} has empty 'completed_event_name'",
                    index
                ).into());
            }
        }

        Ok(())
    }
}
