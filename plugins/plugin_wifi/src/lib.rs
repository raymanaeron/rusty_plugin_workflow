extern crate plugin_core;

mod network_info;
use network_info::{ NetworkInfo, to_json };

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::process::Command;
use std::collections::HashMap;
use std::io::Cursor;
use hex;

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

#[cfg(target_os = "windows")]
fn verify_wifi_connection(ssid: &str, interface: &str) -> bool {
    // Give it a moment to establish connection
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    let status = Command::new("netsh")
        .args(["wlan", "show", "interface", "name", interface])
        .output();
        
    match status {
        Ok(output) => {
            let status_str = String::from_utf8_lossy(&output.stdout);
            // Check both SSID and connection status
            status_str.contains(ssid) && status_str.contains("State") && status_str.contains("connected")
        }
        Err(_) => false
    }
}

#[cfg(target_os = "windows")]
fn connect_wifi_impl(ssid: &str, password: &str) -> bool {
    println!("[plugin_wifi] Attempting to connect to {} on Windows...", ssid);
    
    // Get interface name first
    let interfaces = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output();

    let interface_name = match interfaces {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output_str.lines().find(|l| l.contains("Name")) {
                if let Some(name) = line.split(":").nth(1) {
                    name.trim().to_string()
                } else {
                    println!("[plugin_wifi] No wireless interface found");
                    return false;
                }
            } else {
                println!("[plugin_wifi] No wireless interface found");
                return false;
            }
        }
        Err(e) => {
            println!("[plugin_wifi] Failed to get interfaces: {}", e);
            return false;
        }
    };

    // Delete existing profile if it exists
    let _ = Command::new("netsh")
        .args(["wlan", "delete", "profile", "name", ssid])
        .output();

    // Create XML profile with proper formatting
    let profile_xml = format!(r#"<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>{}</name>
    <SSIDConfig>
        <SSID>
            <hex>{}</hex>
            <name>{}</name>
        </SSID>
        <nonBroadcast>false</nonBroadcast>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>manual</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>WPA2PSK</authentication>
                <encryption>AES</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
            <sharedKey>
                <keyType>passPhrase</keyType>
                <protected>false</protected>
                <keyMaterial>{}</keyMaterial>
            </sharedKey>
        </security>
    </MSM>
    <MacRandomization xmlns="http://www.microsoft.com/networking/WLAN/profile/v3">
        <enableRandomization>false</enableRandomization>
    </MacRandomization>
</WLANProfile>"#,
        ssid,
        hex::encode(ssid.as_bytes()),
        ssid,
        password
    );

    println!("[plugin_wifi] Generated profile XML:\n{}", profile_xml);

    // Write profile to temporary file with proper path
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("{}_{}.xml", ssid, std::process::id()));
    let temp_path_str = temp_path.to_string_lossy();

    if let Err(e) = std::fs::write(&temp_path, profile_xml) {
        println!("[plugin_wifi] Failed to write profile: {}", e);
        return false;
    }

    // Add profile with proper parameter format
    let profile_cmd = Command::new("netsh")
        .args([
            "wlan", 
            "add", 
            "profile", 
            &format!("filename=\"{}\"", temp_path_str),
            &format!("interface=\"{}\"", interface_name),
            "user=current"
        ])
        .output();

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    match profile_cmd {
        Ok(output) => {
            println!("[plugin_wifi] Profile command output: {}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                println!("[plugin_wifi] Failed to add WiFi profile: {}", 
                    String::from_utf8_lossy(&output.stderr));
                return false;
            }

            println!("[plugin_wifi] WiFi profile added successfully");
            
            // Try connecting multiple times
            for attempt in 1..=3 {
                println!("[plugin_wifi] Connection attempt {} of 3", attempt);
                
                let connect = Command::new("netsh")
                    .args([
                        "wlan", 
                        "connect", 
                        &format!("name=\"{}\"", ssid),
                        &format!("interface=\"{}\"", interface_name)
                    ])
                    .output();
                    
                match connect {
                    Ok(output) => {
                        println!("[plugin_wifi] Connection output: {}", String::from_utf8_lossy(&output.stdout));
                        if output.status.success() {
                            if verify_wifi_connection(ssid, &interface_name) {
                                println!("[plugin_wifi] Successfully connected and verified connection to {}", ssid);
                                return true;
                            } else {
                                println!("[plugin_wifi] Connection reported success but verification failed");
                            }
                        }
                        if !output.stderr.is_empty() {
                            println!("[plugin_wifi] Attempt {} failed: {}", 
                                attempt,
                                String::from_utf8_lossy(&output.stderr));
                        }
                    }
                    Err(e) => {
                        println!("[plugin_wifi] Connection error on attempt {}: {}", attempt, e);
                    }
                }
                
                if attempt < 3 {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }

            // If all attempts failed, clean up the profile
            let _ = Command::new("netsh")
                .args(["wlan", "delete", "profile", "name", ssid])
                .output();
        }
        Err(e) => {
            println!("[plugin_wifi] Failed to create profile: {}", e);
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn connect_wifi_impl(ssid: &str, password: &str) -> bool {
    println!("[plugin_wifi] Attempting to connect to {} on Linux...", ssid);
    
    // Try nmcli first (NetworkManager)
    let nmcli = Command::new("nmcli")
        .args(["device", "wifi", "connect", ssid, "password", password])
        .output();
    
    match nmcli {
        Ok(output) => {
            if output.status.success() {
                println!("[plugin_wifi] Successfully connected using NetworkManager");
                return true;
            }
            println!("[plugin_wifi] NetworkManager connection failed: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            println!("[plugin_wifi] NetworkManager error: {}", e);
        }
    }

    println!("[plugin_wifi] Falling back to wpa_supplicant...");
    
    // Fallback to wpa_supplicant
    match Command::new("wpa_passphrase")
        .args([ssid, password])
        .output() 
    {
        Ok(output) => {
            if !output.status.success() {
                println!("[plugin_wifi] wpa_passphrase failed: {}", 
                    String::from_utf8_lossy(&output.stderr));
                return false;
            }

            match std::str::from_utf8(&output.stdout) {
                Ok(config) => {
                    match std::fs::write("/tmp/wpa_supplicant.conf", config) {
                        Ok(_) => {
                            match Command::new("wpa_supplicant")
                                .args(["-B", "-i", "wlan0", "-c", "/tmp/wpa_supplicant.conf"])
                                .output() 
                            {
                                Ok(output) => {
                                    if output.status.success() {
                                        println!("[plugin_wifi] Successfully connected using wpa_supplicant");
                                        true
                                    } else {
                                        println!("[plugin_wifi] wpa_supplicant connection failed: {}", 
                                            String::from_utf8_lossy(&output.stderr));
                                        false
                                    }
                                }
                                Err(e) => {
                                    println!("[plugin_wifi] wpa_supplicant error: {}", e);
                                    false
                                }
                            }
                        }
                        Err(e) => {
                            println!("[plugin_wifi] Failed to write config file: {}", e);
                            false
                        }
                    }
                }
                Err(e) => {
                    println!("[plugin_wifi] Failed to parse wpa_passphrase output: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            println!("[plugin_wifi] wpa_passphrase error: {}", e);
            false
        }
    }
}

#[cfg(target_os = "macos")]
fn connect_wifi_impl(ssid: &str, password: &str) -> bool {
    println!("[plugin_wifi] Attempting to connect to {} on macOS...", ssid);
    
    let output = Command::new("networksetup")
        .args(["-setairportnetwork", "en0", ssid, password])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("[plugin_wifi] Successfully connected to {}", ssid);
                true
            } else {
                println!("[plugin_wifi] Connection failed: {}", 
                    String::from_utf8_lossy(&output.stderr));
                false
            }
        }
        Err(e) => {
            println!("[plugin_wifi] Connection error: {}", e);
            false
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn connect_wifi_impl(_: &str, _: &str) -> bool {
    println!("[plugin_wifi] WiFi connection not available on this platform");
    false
}

fn connect_to_network(ssid: &str, password: &str) -> *mut ApiResponse {
    let success = connect_wifi_impl(ssid, password);
    
    {
        let mut flag = WIFI_CONNECTED.lock().unwrap();
        *flag = success;
    }

    if success {
        println!("[plugin_wifi] Connected to {}", ssid);
        let msg = format!(r#"{{ "message": "Connected to {}" }}"#, ssid);
        json_response(200, &msg)
    } else {
        let msg = format!(r#"{{ "message": "Failed to connect to {}" }}"#, ssid);
        json_response(500, &msg)
    }
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
    // Try scanning multiple times
    for attempt in 1..=3 {
        println!("[plugin_wifi] Scan attempt {} of 3", attempt);
        
        let output = if cfg!(target_os = "windows") {
            // Add rescan parameter
            Command::new("netsh")
                .args(["wlan", "show", "networks", "mode=bssid"])
                .output()
        } else if cfg!(target_os = "linux") {
            // Force rescan on Linux
            let _ = Command::new("nmcli").args(["dev", "wifi", "rescan"]).output();
            std::thread::sleep(std::time::Duration::from_secs(1));
            Command::new("nmcli")
                .args(["-t", "-f", "SSID,BSSID,SIGNAL,CHAN,SECURITY,FREQ,FLAGS", "dev", "wifi"])
                .output()
        } else if cfg!(target_os = "macos") {
            Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
                .args(["-s", "-x"])
                .output()
        } else {
            return ptr::null_mut();
        };

        match output {
            Ok(out) => {
                let networks = parse_scan_output(&out.stdout);
                if !networks.is_empty() {
                    let boxed = networks.into_boxed_slice();
                    unsafe {
                        *out_count = boxed.len();
                    }
                    return Box::into_raw(boxed) as *mut NetworkInfo;
                }
            }
            Err(e) => println!("[plugin_wifi] Scan attempt {} failed: {}", attempt, e),
        }

        if attempt < 3 {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    unsafe {
        *out_count = 0;
    }
    ptr::null_mut()
}

// Helper function to parse scan output
fn parse_scan_output(output: &[u8]) -> Vec<NetworkInfo> {
    let raw_output = String::from_utf8_lossy(output);
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
        match plist::Value::from_reader_xml(Cursor::new(output)) {  // Pass output bytes directly
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
            Err(e) => {
                println!("[plugin_wifi] Failed to parse plist: {}", e);
            },
            _ => {}
        }
    }

    let networks: Vec<NetworkInfo> = unique_networks.into_iter()
        .map(|(_, (network, _))| network)
        .collect();

    networks
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
