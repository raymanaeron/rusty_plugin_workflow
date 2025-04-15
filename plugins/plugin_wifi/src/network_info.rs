use std::os::raw::{c_char, c_int, c_float};
use std::ffi::CStr;

/// Represents a single Wi-Fi network discovered during a scan operation.
///
/// This structure is used as part of the plugin's low-level API (`scan()`), which
/// returns a pointer to an array of `NetworkInfo` structs. It is a plugin-specific
/// data model and should remain within the `plugin_wifi` domain.
///
/// Each field is populated from a system-specific Wi-Fi scanning command (`netsh` on Windows,
/// `nmcli` on Linux), and is passed over FFI from the plugin to the engine or used
/// directly within the plugin’s internal API handlers.
///
/// ### Field Descriptions
///
/// - `ssid`: The network name (e.g., `"HomeWiFi"`).
/// - `bssid`: The MAC address of the access point (e.g., `"00:1A:2B:3C:4D:5E"`).
/// - `signal`: Signal strength (typically in percentage or dBm depending on platform).
/// - `channel`: Channel number the network is operating on.
/// - `security`: Description of the security protocol (e.g., `"WPA2"`, `"Open"`).
/// - `frequency`: Operating frequency in MHz (e.g., `2412.0` for 2.4GHz, `5180.0` for 5GHz).
///
/// ### FFI Considerations
///
/// - All string fields (`ssid`, `bssid`, `security`) must be null-terminated C strings.
/// - Memory allocated for these strings should be freed during plugin cleanup,
///   especially if the struct is returned to the engine or serialized into JSON.
/// - The array returned by `scan()` should be properly boxed and its count returned
///   via an out-pointer to ensure correct length.
///
#[repr(C)]
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    /// Network SSID (e.g., "MyWiFi").
    pub ssid: *const c_char,

    /// BSSID (MAC address) of the access point.
    pub bssid: *const c_char,

    /// Signal strength in percentage or dBm, depending on platform.
    pub signal: c_int,

    /// Wi-Fi channel number (e.g., 6 for 2.4GHz, 36 for 5GHz).
    pub channel: c_int,

    /// Security protocol used (e.g., "WPA2", "Open").
    pub security: *const c_char,

    /// Operating frequency in MHz (e.g., 2412.0 for 2.4GHz).
    pub frequency: c_float,
}

/// We need this structure to serialize the data on axum route handler
#[derive(serde::Serialize)]
pub struct NetworkInfoJson {
    ssid: String,
    bssid: String,
    signal: i32,
    channel: i32,
    security: String,
    frequency: f32,
}

// This is for the NetworkInfo struct
pub fn to_json(net: &NetworkInfo) -> NetworkInfoJson {
    NetworkInfoJson {
        ssid: unsafe { CStr::from_ptr(net.ssid).to_string_lossy().into_owned() },
        bssid: unsafe { CStr::from_ptr(net.bssid).to_string_lossy().into_owned() },
        signal: net.signal,
        channel: net.channel,
        security: unsafe { CStr::from_ptr(net.security).to_string_lossy().into_owned() },
        frequency: net.frequency,
    }
}