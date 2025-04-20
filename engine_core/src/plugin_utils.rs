use std::path::PathBuf;
use std::fs;
use std::io::Read;
use std::io::Write;

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

/// Downloads a plugin binary from an S3 HTTPS URL and stores it in the exe folder.
/// Returns the final local path to the copied file.
pub fn download_plugin_from_s3(url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let response = ureq::get(url).call()?;
    if response.status() != 200 {
        return Err(format!("HTTP GET failed with status {}", response.status()).into());
    }

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let mut exe_path = std::env::current_exe()?;
    exe_path.pop();

    let filename = url.split('/').last().ok_or("Invalid URL filename")?;
    let final_path = exe_path.join(filename);

    let mut file = fs::File::create(&final_path)?;
    file.write_all(&bytes)?;

    Ok(final_path)
}