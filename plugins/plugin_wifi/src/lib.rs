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

    let networks = vec![
        NetworkInfo {
            ssid: CString::new("DummyNet").unwrap().into_raw(),
            bssid: CString::new("00:00:00:00:00:00").unwrap().into_raw(),
            signal: -50,
            channel: 1,
            security: CString::new("WPA2").unwrap().into_raw(),
            frequency: 2.437,
        }
    ];

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
