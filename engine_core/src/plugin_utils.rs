/// Returns the filename of the plugin based on platform (e.g., libfoo.so, foo.dll, libfoo.dylib)
pub fn resolve_plugin_filename(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.dll", name)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", name)
    } else {
        format!("lib{}.so", name)
    }
}

/// Combines the folder and resolved filename into a full path.
pub fn resolve_plugin_binary_path(folder: &str, name: &str) -> String {
    let filename = resolve_plugin_filename(name);

    let mut base_path = std::path::PathBuf::from(folder);
    base_path.push(filename);

    base_path.to_string_lossy().into_owned()
}
