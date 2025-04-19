use serde::Deserialize;

/// Represents metadata for a dynamically loaded plugin as defined in the execution plan.
#[derive(Debug, Deserialize)]
pub struct PluginMetadata {
    /// Internal name of the plugin (used to match the shared library file).
    pub full_name: String,

    /// Logical identifier used for routing and workflow naming.
    pub plugin_route: String,

    /// Semantic version of the plugin (e.g., 1.0.0, 2.1.5).
    pub version: String,

    /// Location type indicating how to load the plugin.
    /// - "local": load from local file path
    /// - "s3": download compiled binary from an S3 URL
    pub plugin_location_type: String,

    /// Actual path or URL of the plugin binary.
    /// - For local: "./plugin_wifi.so"
    /// - For s3: "https://bucket-name.s3.amazonaws.com/plugin_wifi.dylib"
    pub plugin_location_path: String,

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
}

/// Default value for `visible_in_ui` field (true).
fn default_visible_in_ui() -> bool {
    true
}
