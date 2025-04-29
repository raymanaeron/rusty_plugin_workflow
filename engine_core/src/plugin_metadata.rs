use serde::Deserialize;

/// Represents metadata for a dynamically loaded plugin as defined in the execution plan.
#[derive(Debug, Deserialize)]
pub struct PluginMetadata {
    /// Plugin binary name (used to resolve OS-specific filename).
    pub name: String,

    /// Logical identifier used for routing and workflow naming.
    pub plugin_route: String,

    /// Semantic version of the plugin (e.g., 1.0.0, 2.1.5).
    pub version: String,

    /// Location type indicating how to load the plugin.
    /// - "local": load from local file path
    /// - "s3": download compiled binary from an S3 URL
    pub plugin_location_type: String,

    /// Folder where the plugin resides (e.g., "./", or an S3 URL prefix)
    pub plugin_base_path: String,

    /// Engineering team name that owns this plugin.
    pub team_name: String,

    /// Email for engineering support.
    pub engineering_contact_email: String,

    /// Email for runtime/operations support.
    pub operation_contact_email: String,

    /// Whether to run the plugin asynchronously via `run_async`. Default: false.
    #[serde(default)]
    pub run_async: bool,

    /// Whether to show this plugin in the UI (progress bar, logs). Default: true.
    #[serde(default = "default_visible_in_ui")]
    pub visible_in_ui: bool,

    /// Optional description of the plugin’s purpose or function.
    #[serde(default)]
    pub plugin_description: String,

    /// Specify this in the toml so that the engine knows when to run this plugin.
    pub run_after_event_name: Option<String>,

    /// Specify this in the toml so that the engine knows that you are done
    pub completed_event_name: Option<String>,
}

/// Default value for `visible_in_ui` field (true).
fn default_visible_in_ui() -> bool {
    true
}

impl PluginMetadata {
    pub fn resolved_local_path(&self) -> String {
        use crate::plugin_utils::resolve_plugin_binary_path;
        resolve_plugin_binary_path(&self.plugin_base_path, &self.name)
    }
}
