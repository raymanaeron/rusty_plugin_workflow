extern crate plugin_core;

mod network_info;
use network_info::{ NetworkInfo, to_json };

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::process::Command;
use std::collections::HashMap;
use std::io::Cursor;

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
        *flag = false; // Reset the flag first to simulate a fresh connection
    }

    // Simulate connection delay or logic here...
    std::thread::sleep(std::time::Duration::from_millis(200)); // Optional

    {
        let mut flag = WIFI_CONNECTED.lock().unwrap();
        *flag = true; // Set connected flag
    }

    println!("[plugin_wifi] Connected to {}", ssid);
    let msg = format!(r#"{{ "message": "Connected to {}" }}"#, ssid);
    json_response(200, &msg)
}


extern "C" fn on_complete() -> *mut ApiResponse {
    let connected = *WIFI_CONNECTED.lock().unwrap();
    println!("[plugin_wifi] on_complete: connected = {}", connected);

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
            .args(["-s", "-x"])
            .output()
    } else {
        return ptr::null_mut();
    };

    let raw_output = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let mut unique_networks: HashMap<String, (NetworkInfo, i32)> = HashMap::new();

    if cfg!(target_os = "linux") {
        for line in raw_output.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() < 6 {
                continue;
            }

            let ssid = fields[0];
            let signal = fields[2].parse::<i32>().unwrap_or(0);

            // Only keep the strongest signal for each SSID
            if let Some(&(_, existing_signal)) = unique_networks.get(ssid) {
                if existing_signal < signal {
                    let bssid = CString::new(fields[1]).unwrap_or_default().into_raw();
                    let channel = fields[3].parse::<i32>().unwrap_or(0);
                    let security = CString::new(fields[4]).unwrap_or_default().into_raw();
                    let frequency = fields[5].parse::<f32>().unwrap_or(0.0);
                    let ssid = CString::new(ssid).unwrap_or_default().into_raw();

                    unique_networks.insert(fields[0].to_string(), (NetworkInfo {
                        ssid,
                        bssid,
                        signal,
                        channel,
                        security,
                        frequency,
                    }, signal));
                }
            } else {
                let bssid = CString::new(fields[1]).unwrap_or_default().into_raw();
                let channel = fields[3].parse::<i32>().unwrap_or(0);
                let security = CString::new(fields[4]).unwrap_or_default().into_raw();
                let frequency = fields[5].parse::<f32>().unwrap_or(0.0);
                let ssid = CString::new(ssid).unwrap_or_default().into_raw();

                unique_networks.insert(fields[0].to_string(), (NetworkInfo {
                    ssid,
                    bssid,
                    signal,
                    channel,
                    security,
                    frequency,
                }, signal));
            }
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
                    // Check if we already have this SSID with a stronger signal
                    if let Some(&(_, existing_signal)) = unique_networks.get(&current_ssid) {
                        if existing_signal >= current_signal {
                            continue;
                        }
                    }

                    let bssid = CString::new(bssid_str.trim()).unwrap_or_default().into_raw();
                    let ssid = CString::new(current_ssid.clone()).unwrap_or_default().into_raw();
                    let security = CString::new(current_security.clone())
                        .unwrap_or_default()
                        .into_raw();

                    unique_networks.insert(current_ssid.clone(), (NetworkInfo {
                        ssid,
                        bssid,
                        signal: current_signal,
                        channel: 0,
                        security,
                        frequency: 0.0,
                    }, current_signal));
                }
            }
        }
    } else if cfg!(target_os = "macos") {
        match plist::Value::from_reader_xml(Cursor::new(&raw_output)) {
            Ok(plist::Value::Dictionary(dict)) => {
                if let Some(plist::Value::Array(networks)) = dict.get("wireless networks") {
                    for network in networks {
                        if let plist::Value::Dictionary(network) = network {
                            let ssid = network.get("SSID_STR")
                                .and_then(|v| v.as_string())
                                .unwrap_or("").to_string();
                            
                            let signal = network.get("RSSI")
                                .and_then(|v| v.as_signed_integer())
                                .map(|v| v as i32)
                                .unwrap_or(0);

                            let bssid = network.get("BSSID")
                                .and_then(|v| v.as_string())
                                .unwrap_or("").to_string();

                            let channel = network.get("CHANNEL")
                                .and_then(|v| v.as_signed_integer())
                                .map(|v| v as i32)
                                .unwrap_or(0);

                            let security = network.get("WPA_IE")
                                .map(|_| "WPA")
                                .or_else(|| network.get("RSN_IE").map(|_| "WPA2"))
                                .unwrap_or("NONE");

                            if !ssid.is_empty() {
                                let ssid_cstr = CString::new(ssid.clone()).unwrap_or_default().into_raw();
                                let bssid_cstr = CString::new(bssid).unwrap_or_default().into_raw();
                                let security_cstr = CString::new(security).unwrap_or_default().into_raw();

                                unique_networks.insert(ssid, (NetworkInfo {
                                    ssid: ssid_cstr,
                                    bssid: bssid_cstr,
                                    signal,
                                    channel,
                                    security: security_cstr,
                                    frequency: 0.0,
                                }, signal));
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }

    let networks: Vec<NetworkInfo> = unique_networks.into_iter()
        .map(|(_, (network, _))| network)
        .collect();

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
