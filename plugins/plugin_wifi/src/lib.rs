extern crate plugin_core;

mod network_info;
use network_info::{ NetworkInfo, to_json };

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::process::Command;

use plugin_core::*;
use plugin_core::resource_utils::static_resource;
use plugin_core::response_utils::*;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static WIFI_CONNECTED: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

#[ctor::ctor]
fn on_load() {
    println!("[plugin_wifi] >>> LOADED");
}

extern "C" fn run(ctx: *const PluginContext) {
    println!("[plugin_wifi] - run");
    println!("[plugin_wifi] FINGERPRINT: run = {:p}", run as *const ());

    if ctx.is_null() {
        eprintln!("PluginContext is null");
        return;
    }

    unsafe {
        let config_cstr = CStr::from_ptr((*ctx).config);
        println!("WiFi Plugin running with config: {}", config_cstr.to_string_lossy());
    }
}

extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("wifi/web").unwrap().into_raw()
}

extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];
    let slice = static_resource("network", &METHODS);
    unsafe { *out_len = slice.len(); }
    slice.as_ptr()
}

extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let request = &*req;
        let path = if request.path.is_null() {
            "<null>"
        } else {
            CStr::from_ptr(request.path).to_str().unwrap_or("<invalid>")
        };

        match request.method {
            HttpMethod::Get if path == "network" => {
                let mut count: usize = 0;
                let result_ptr = scan(&mut count);

                let json = if result_ptr.is_null() || count == 0 {
                    "[]".to_string()
                } else {
                    let results = std::slice::from_raw_parts(result_ptr, count);
                    let objects: Vec<_> = results.iter().map(to_json).collect();
                    serde_json::to_string(&objects).unwrap_or("[]".into())
                };

                return json_response(200, &json);
            }

            HttpMethod::Post if path == "network" => {
                let body = std::slice::from_raw_parts(request.body_ptr, request.body_len);
                let body_str = std::str::from_utf8(body).unwrap_or("");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(body_str) {
                    let ssid = json.get("ssid").and_then(|v| v.as_str()).unwrap_or("");
                    let password = json.get("password").and_then(|v| v.as_str()).unwrap_or("");
                    return connect_to_network(ssid, password);
                }
                return error_response(400, "Invalid JSON payload");
            }

            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}

fn connect_to_network(ssid: &str, _password: &str) -> *mut ApiResponse {
    {
        let mut flag = WIFI_CONNECTED.lock().unwrap();
        *flag = true;  // Mark as connected
    }
    let msg = format!(r#"{{ "message": "Connected to {}" }}"#, ssid);
    json_response(200, &msg)
}

extern "C" fn on_complete() -> *mut ApiResponse {
    let connected = *WIFI_CONNECTED.lock().unwrap();
    if connected {
        json_response(200, r#"{ "message": "WiFi Connected" }"#)
    } else {
        json_response(204, r#"{ "message": "WiFi not connected" }"#)
    }
}


extern "C" fn scan(out_count: *mut usize) -> *mut NetworkInfo {
    let output = if cfg!(target_os = "windows") {
        Command::new("netsh").args(["wlan", "show", "networks", "mode=bssid"]).output()
    } else if cfg!(target_os = "linux") {
        Command::new("nmcli").args(["-t", "-f", "SSID,BSSID,SIGNAL,CHAN,SECURITY,FREQ", "dev", "wifi"]).output()
    } else if cfg!(target_os = "macos") {
        Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
            .arg("-s")
            .output()
    } else {
        return ptr::null_mut();
    };

    let raw_output = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let mut networks = Vec::new();

    if cfg!(target_os = "linux") {
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
                    current_signal = percent_str
                        .trim()
                        .trim_end_matches('%')
                        .parse::<i32>()
                        .unwrap_or(0);
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
                    let security = CString::new(current_security.clone())
                        .unwrap_or_default()
                        .into_raw();

                    networks.push(NetworkInfo {
                        ssid,
                        bssid,
                        signal: current_signal,
                        channel: 0,
                        security,
                        frequency: 0.0,
                    });
                }
            }
        }
    } else if cfg!(target_os = "macos") {
        for (i, line) in raw_output.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let ssid = CString::new(parts[0]).unwrap_or_default().into_raw();
            let bssid = CString::new(parts[1]).unwrap_or_default().into_raw();
            let signal = parts[2].parse::<i32>().unwrap_or(0);
            let channel = parts[3].parse::<i32>().unwrap_or(0);
            let security = CString::new(parts[5..].join(" "))
                .unwrap_or_default()
                .into_raw();

            networks.push(NetworkInfo {
                ssid,
                bssid,
                signal,
                channel,
                security,
                frequency: 0.0,
            });
        }
    }

    let boxed = networks.into_boxed_slice();
    unsafe {
        *out_count = boxed.len();
    }

    Box::into_raw(boxed) as *mut NetworkInfo
}

extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

extern "C" fn null_workflow(_req: *const ApiRequest) -> *mut ApiResponse {
    std::ptr::null_mut()
}

extern "C" fn null_progress() -> *mut ApiResponse {
    std::ptr::null_mut()
}

declare_plugin!(
    "plugin_wifi",
    "wifi",
    run,
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup,
    null_workflow,
    null_progress,
    on_complete
);
