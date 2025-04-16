// plugins/plugin_wifi/src/lib.rs
mod network_info;
use network_info::{NetworkInfo, to_json};
use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;
use plugin_core::{ApiRequest, ApiResponse, HttpMethod, Resource, Plugin, PluginContext};
use plugin_core::{method_not_allowed, cleanup_response};
use std::ptr;

// This function returns the name of the plugin
extern "C" fn name() -> *const c_char {
    CString::new("wifi").unwrap().into_raw()
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

fn connect_to_network(ssid: &str, password: &str) -> *mut ApiResponse {
    println!("Connecting to SSID: {}, Password: {}", ssid, password);

    let message = format!(r#"{{ "message": "Connected to {}" }}"#, ssid);
    let body = message.into_bytes();
    let body_len = body.len();
    let body_ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;

    let content_type = CString::new("application/json").unwrap();
    
    let response = Box::new(ApiResponse {
        status: 200,
        headers: ptr::null(),     // optional
        header_count: 0,
        body_ptr,
        body_len,
        content_type: content_type.into_raw(),
    });

    Box::into_raw(response)
}

// This function returns the static content folder path
extern "C" fn get_static_content_path() -> *const c_char {
    CString::new("wifi/web").unwrap().into_raw()
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

/*
extern "C" fn method_not_allowed(method: HttpMethod, resource: *const c_char) -> *const c_char {
    let method_str = match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
    };

    let res_str = unsafe {
        if resource.is_null() {
            "<unknown>"
        } else {
            std::ffi::CStr::from_ptr(resource).to_str().unwrap_or("<invalid>")
        }
    };

    let msg = format!("Method {} not allowed on {}", method_str, res_str);
    CString::new(msg).unwrap().into_raw()
}*/

#[no_mangle]
pub extern "C" fn get_api_resources(count: *mut usize) -> *const Resource {
    // Static backing for the "network" path
    // static mut NETWORK_PATH: Option<CString> = None;
    static NETWORK_PATH: Mutex<Option<CString>> = Mutex::new(None);

    // Static array holding the single supported method
    // static SUPPORTED_METHODS: [HttpMethod; 1] = [HttpMethod::Get];
    // Support both GET and POST now
    static SUPPORTED_METHODS: [HttpMethod; 2] = [HttpMethod::Get, HttpMethod::Post];

    // Static boxed array of one Resource
    // tatic mut RESOURCE_LIST: Option<Box<[Resource]>> = None;
    static RESOURCE_LIST: Mutex<Option<Box<[Resource]>>> = Mutex::new(None);

    let mut network_path_lock = NETWORK_PATH.lock().unwrap();
    let mut resource_list_lock = RESOURCE_LIST.lock().unwrap();

    *network_path_lock = Some(CString::new("network").unwrap());

    *resource_list_lock = Some(Box::new([Resource::new(
        network_path_lock.as_ref().unwrap().as_ptr(),
        SUPPORTED_METHODS.as_ptr(),
    )]));

    if !count.is_null() {
        unsafe {
            *count = resource_list_lock.as_ref().unwrap().len();
        }
    }

    resource_list_lock.as_ref().unwrap().as_ptr()
}

#[no_mangle]
pub extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
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
            HttpMethod::Get => {
                if path == "network" {
                    // Call scan() directly
                    let mut count: usize = 0;
                    let result_ptr = scan(&mut count as *mut usize);

                    let response_json = if result_ptr.is_null() || count == 0 {
                        "[]".to_string()
                    } else {
                        let results = std::slice::from_raw_parts(result_ptr, count);
                        let json_objects: Vec<_> = results.iter().map(to_json).collect();
                        serde_json::to_string(&json_objects).unwrap_or("[]".to_string())
                    };

                    let body_bytes = response_json.into_bytes();
                    let len = body_bytes.len();
                    let ptr = Box::into_raw(body_bytes.into_boxed_slice()) as *const u8;

                    let content_type = CString::new("application/json").unwrap().into_raw();

                    let response = Box::new(ApiResponse {
                        headers: ptr::null(),
                        header_count: 0,
                        content_type,
                        status: 200,
                        body_ptr: ptr,
                        body_len: len,
                    });

                    return Box::into_raw(response);
                }
            }

            // Handle POST requests
            HttpMethod::Post => {
                if path == "network" {
                    let body_slice = 
                        std::slice::from_raw_parts(request.body_ptr, request.body_len);
                    
                    let body_str = std::str::from_utf8(body_slice).unwrap_or("");
                    let parsed: Result<serde_json::Value, _> = serde_json::from_str(body_str);
        
                    if let Ok(json) = parsed {
                        let ssid = json.get("ssid").and_then(|v| v.as_str()).unwrap_or("");
                        let password = json.get("password").and_then(|v| v.as_str()).unwrap_or("");
        
                        return connect_to_network(ssid, password);
                    } else {
                        return plugin_core::error_response(400, "Invalid JSON payload");
                    }
                }
            }

            // All other methods
            _ => {
                let err_ptr = method_not_allowed(request.method, request.path);

                let body = CStr::from_ptr(err_ptr).to_bytes().to_vec();
                let len = body.len();
                let ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;

                let content_type = CString::new("text/plain").unwrap().into_raw();

                let response = Box::new(ApiResponse {
                    headers: ptr::null(),
                    header_count: 0,
                    content_type,
                    status: 405,
                    body_ptr: ptr,
                    body_len: len,
                });

                return Box::into_raw(response);
            }
        }

        // No matching path
        let msg = CString::new("Not Found").unwrap();
        let body = msg.as_bytes().to_vec();
        let len = body.len();
        let ptr = Box::into_raw(body.into_boxed_slice()) as *const u8;
        let content_type = CString::new("text/plain").unwrap().into_raw();

        let response = Box::new(ApiResponse {
            headers: ptr::null(),
            header_count: 0,
            content_type,
            status: 404,
            body_ptr: ptr,
            body_len: len,
        });

        Box::into_raw(response)
    }
}

#[no_mangle]
pub extern "C" fn cleanup(resp: *mut ApiResponse) {
    cleanup_response(resp);
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *const Plugin {
    &Plugin {
        name,
        run,
        get_static_content_path,
        get_api_resources,
        handle_request,
        cleanup,
    }
}