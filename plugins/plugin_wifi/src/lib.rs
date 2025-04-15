// plugins/plugin_wifi/src/lib.rs

use plugin_core::{Plugin, PluginContext, NetworkInfo};
use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;
use axum::{Router, routing::get, Json};

// This is the plugin's static content folder
static ROUTER: Mutex<Option<Box<Router>>> = Mutex::new(None);

/// We need this structure to serialize the data on axum route handler
#[derive(serde::Serialize)]
struct NetworkInfoJson {
    ssid: String,
    bssid: String,
    signal: i32,
    channel: i32,
    security: String,
    frequency: f32,
}

// This is for the NetworkInfo struct
fn to_json(net: &NetworkInfo) -> NetworkInfoJson {
    NetworkInfoJson {
        ssid: unsafe { CStr::from_ptr(net.ssid).to_string_lossy().into_owned() },
        bssid: unsafe { CStr::from_ptr(net.bssid).to_string_lossy().into_owned() },
        signal: net.signal,
        channel: net.channel,
        security: unsafe { CStr::from_ptr(net.security).to_string_lossy().into_owned() },
        frequency: net.frequency,
    }
}

// This is the route handler for the /scan endpoint
async fn scan_handler() -> Json<Vec<NetworkInfoJson>> {
    unsafe {
        let mut count: usize = 0;
        let result_ptr = scan(&mut count as *mut usize);
        if result_ptr.is_null() || count == 0 {
            return Json(vec![]);
        }

        let results: &[NetworkInfo] = std::slice::from_raw_parts(result_ptr, count);
        let json_results = results.iter().map(to_json).collect();
        Json(json_results)
    }
}

// This function returns the name of the plugin
extern "C" fn name() -> *const c_char {
    CString::new("WiFi Plugin").unwrap().into_raw()
}

// This function is called when the plugin is loaded
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

// This function creates the API route for the plugin
extern "C" fn get_api_route() -> *mut Router {
    let router = Router::new().route("/scan", get(scan_handler));
    let boxed = Box::new(router);

    // Lock the mutex to safely access the static
    let mut router_lock = ROUTER.lock().unwrap();
    *router_lock = Some(boxed);

    router_lock.as_mut().unwrap().as_mut() as *mut Router
}

// This function returns the static content folder path
extern "C" fn get_static_content_route() -> *const c_char {
    CString::new("plugins/plugin_wifi/web").unwrap().into_raw()
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
pub extern "C" fn create_plugin() -> *const Plugin {
    &Plugin {
        name,
        run,
        get_api_route,
        get_static_content_route
    }
}
