// plugins/plugin_wifi/src/lib.rs

use plugin_api::{PluginApi, PluginContext, NetworkInfo};
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" fn name() -> *const c_char {
    CString::new("WiFi Plugin").unwrap().into_raw()
}

extern "C" fn run(ctx: *const PluginContext) {
    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config_cstr = std::ffi::CStr::from_ptr((*ctx).config);
        println!("WiFi Plugin running with config: {}", config_cstr.to_string_lossy());
    }
}

#[no_mangle]
pub extern "C" fn scan(out_count: *mut usize) -> *mut NetworkInfo {
    use std::process::Command;
    use std::ffi::CString;
    use std::ptr;

    let output = if cfg!(target_os = "windows") {
        Command::new("netsh")
            .args(["wlan", "show", "networks", "mode=bssid"])
            .output()
    } else if cfg!(target_os = "linux") {
        Command::new("nmcli")
            .args(["-t", "-f", "SSID,BSSID,SIGNAL,CHAN,SECURITY,FREQ", "dev", "wifi"])
            .output()
    } else {
        eprintln!("Unsupported OS for Wi-Fi scan.");
        return ptr::null_mut();
    };

    let raw_output = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            eprintln!("Failed to run scan command: {}", e);
            return ptr::null_mut();
        }
    };

    println!("[plugin_wifi] Raw scan output:\n{}", raw_output);

    let mut networks = Vec::new();

    if cfg!(target_os = "linux") {
        // Each line: SSID:BSSID:SIGNAL:CHAN:SECURITY:FREQ
        for line in raw_output.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() < 6 {
                continue;
            }

            let ssid = CString::new(fields[0]).unwrap_or_default().into_raw();
            let bssid = CString::new(fields[1]).unwrap_or_default().into_raw();
            let signal = fields[2].parse::<i32>().unwrap_or(0);
            let channel = fields[3].parse::<i32>().unwrap_or(0);
            let security = CString::new(fields[4]).unwrap_or_default().into_raw();
            let frequency = fields[5].parse::<f32>().unwrap_or(0.0);

            networks.push(NetworkInfo {
                ssid,
                bssid,
                signal,
                channel,
                security,
                frequency,
            });
        }
    } else if cfg!(target_os = "windows") {
        let mut current_ssid = String::new();
        let mut current_signal = 0;
        let mut current_security = String::new();

        for line in raw_output.lines() {
            let line = line.trim();
            if line.starts_with("SSID") && line.contains(":") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                current_ssid = parts[1].trim().to_string();
            } else if line.starts_with("Signal") {
                if let Some(percent_str) = line.split(':').nth(1) {
                    current_signal = percent_str.trim().trim_end_matches('%').parse::<i32>().unwrap_or(0);
                }
            } else if line.starts_with("Authentication") {
                if let Some(sec) = line.split(':').nth(1) {
                    current_security = sec.trim().to_string();
                }
            } else if line.starts_with("BSSID") {
                if let Some(bssid_str) = line.split(':').nth(1) {
                    let bssid = bssid_str.trim().to_string();
                    let ssid = CString::new(current_ssid.clone()).unwrap_or_default().into_raw();
                    let bssid = CString::new(bssid).unwrap_or_default().into_raw();
                    let security = CString::new(current_security.clone()).unwrap_or_default().into_raw();

                    networks.push(NetworkInfo {
                        ssid,
                        bssid,
                        signal: current_signal,
                        channel: 0,        // Channel parsing optional, skipped here
                        security,
                        frequency: 0.0,    // Windows netsh does not give frequency
                    });
                }
            }
        }
    }

    let count = networks.len();
    unsafe {
        if !out_count.is_null() {
            *out_count = count;
        }
    }

    let boxed = networks.into_boxed_slice();
    Box::into_raw(boxed) as *mut NetworkInfo
}


#[no_mangle]
pub extern "C" fn create_plugin() -> *const PluginApi {
    &PluginApi {
        name,
        run,
    }
}
