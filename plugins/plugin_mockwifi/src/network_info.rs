use std::os::raw::{c_char, c_int, c_float};
use std::ffi::CStr;

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