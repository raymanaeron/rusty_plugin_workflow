use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use crate::plugin_metadata::PluginMetadata;

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

    let mut base_path = PathBuf::from(folder);
    base_path.push(filename);

    base_path.to_string_lossy().into_owned()
}

/// Resolves the plugin file name (with extension) into the exe directory.
pub fn resolve_plugin_exe_path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let filename = resolve_plugin_filename(name);
    let mut exe_path = std::env::current_exe()?;
    exe_path.pop();
    exe_path.push(filename);
    Ok(exe_path)
}

/// Centralized function to write bytes to a file in the exe directory.
/// Returns the final full path.
fn write_to_exe_dir(filename: &str, content: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut exe_path = std::env::current_exe()?;
    exe_path.pop();
    let final_path = exe_path.join(filename);

    let mut file = fs::File::create(&final_path)?;
    file.write_all(content)?;

    Ok(final_path)
}

/// Downloads a plugin binary from an S3 HTTPS URL and stores it in the exe folder.
/// Returns the final local path to the copied file.
pub fn download_plugin_from_s3(url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let response = ureq::get(url).call();

    if let Err(err) = response {
        return Err(format!("Network error: {}", err).into());
    }

    let response = response.unwrap();
    if response.status() != 200 {
        return Err(format!(
            "HTTP GET failed with status {}",
            response.status()
        ).into());
    }

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    if bytes.is_empty() {
        return Err("Download returned empty content".into());
    }

    let filename = url
        .split('/')
        .last()
        .ok_or("Invalid URL: no filename found")?;

    write_to_exe_dir(filename, &bytes)
}

/// Copies a plugin binary from a known source (UNC or local folder) into the exe directory.
/// Returns the full local path where the file was copied.
pub fn copy_plugin_to_exe_dir(source: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        return Err(format!("Plugin binary not found at: {}", source).into());
    }

    let filename = source_path
        .file_name()
        .ok_or("Missing filename from source path")?
        .to_string_lossy();

    let bytes = fs::read(source_path)?;
    write_to_exe_dir(&filename, &bytes)
}

/// Resolves and prepares the plugin binary locally before load.
/// Handles download or copy depending on plugin_location_type.
pub fn prepare_plugin_binary(plugin: &PluginMetadata, allow_write: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if !allow_write {
        return resolve_plugin_exe_path(&plugin.name);
    }

    match plugin.plugin_location_type.as_str() {
        "local" | "unc" => {
            let resolved_path = plugin.resolved_local_path();
            copy_plugin_to_exe_dir(&resolved_path)
        }
        "s3" => {
            let remote_url = plugin.resolved_local_path();
            download_plugin_from_s3(&remote_url)
        }
        other => Err(format!("Unsupported plugin location type '{}'", other).into()),
    }
}

