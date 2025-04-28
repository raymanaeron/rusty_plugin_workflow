//! WiFi Manager Cross-Platform Module
//! 
//! Provides cross-platform WiFi operations using:
//! - tokio-wifiscanner for WiFi scanning
//! - wifi-rs for WiFi connections
//! 
//! Supports Windows, macOS, and Linux platforms through unified APIs

use std::ffi::{CString, c_char, CStr};
use std::ptr;
use std::thread;
use std::time::Duration;

// For WiFi scanning
use tokio_wifiscanner::{self, Wifi};

// For WiFi connections
use wifi_rs;
use wifi_rs::prelude::*; // Import all traits including Connectivity

use crate::network_info::NetworkInfo;

// Simplified runtime wrapper that handles async correctly
fn run_scan() -> Result<Vec<Wifi>, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        match tokio_wifiscanner::scan().await {
            Ok(networks) => Ok(networks),
            Err(e) => Err(Box::<dyn std::error::Error>::from(e))
        }
    })
}

/// Converts a WiFi security type to a string representation
fn security_type_to_string(security: &str) -> String {
    if security.contains("WPA2") && security.contains("Enterprise") {
        "Enterprise".to_string()
    } else if security.contains("WPA2") {
        "WPA2 Personal".to_string()
    } else if security.contains("WPA3") {
        "WPA3 Personal".to_string()
    } else if security.contains("WPA") {
        "WPA Personal".to_string()
    } else if security.contains("WEP") {
        "WEP".to_string()
    } else if security == "Open" || security == "None" {
        "None".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Calculates a frequency from a channel number
fn channel_to_frequency(channel: u32) -> f32 {
    if channel >= 1 && channel <= 14 {
        // 2.4 GHz band
        (2412 + (channel - 1) * 5) as f32
    } else if channel >= 36 && channel <= 165 {
        // 5 GHz band
        if channel <= 48 {
            (5180 + (channel - 36) * 5) as f32
        } else if channel <= 64 {
            (5260 + (channel - 52) * 5) as f32
        } else if channel <= 144 {
            (5500 + (channel - 100) * 5) as f32
        } else {
            (5745 + (channel - 149) * 5) as f32
        }
    } else {
        0.0 // Unknown channel
    }
}

/// Extract channel number from frequency - commented out since it's unused
#[allow(dead_code)]
fn frequency_to_channel(freq: u32) -> u32 {
    if freq >= 2412 && freq <= 2484 {
        // 2.4 GHz band
        if freq == 2484 {
            14  // Special case for channel 14
        } else {
            (freq - 2412) / 5 + 1
        }
    } else if freq >= 5170 && freq <= 5825 {
        // 5 GHz band
        if freq >= 5745 {
            // Channels 149-165
            (freq - 5745) / 5 + 149
        } else if freq >= 5500 {
            // Channels 100-144
            (freq - 5500) / 5 + 100
        } else if freq >= 5260 {
            // Channels 52-64
            (freq - 5260) / 5 + 52
        } else {
            // Channels 36-48
            (freq - 5180) / 5 + 36
        }
    } else {
        0  // Unknown frequency
    }
}

/// Scans for available WiFi networks using tokio-wifiscanner
pub fn scan(out_count: *mut usize) -> *mut NetworkInfo {
    println!("[plugin_wifi] Starting WiFi scan with tokio-wifiscanner");
    
    for attempt in 1..=3 {
        println!("[plugin_wifi] Scan attempt {} of 3", attempt);
        
        // Use a properly configured async runtime
        match run_scan() {
            Ok(networks) => {
                if networks.is_empty() {
                    println!("[plugin_wifi] No networks found in scan attempt {}", attempt);
                    if attempt < 3 {
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                } else {
                    println!("[plugin_wifi] Found {} networks", networks.len());
                    
                    // Debug output to match documentation example
                    for network in &networks {
                        // Note: Direct field access without Option unwrapping
                        println!(
                            "{} {:15} {:10} {:?} {}",
                            network.mac, network.ssid, 
                            network.channel, network.signal_level, 
                            network.security
                        );
                    }
                    
                    let results = process_scan_results(networks);
                    println!("[plugin_wifi] Processed {} unique networks", results.len());
                    
                    let boxed_results = results.into_boxed_slice();
                    unsafe {
                        *out_count = boxed_results.len();
                    }
                    return Box::into_raw(boxed_results) as *mut NetworkInfo;
                }
            },
            Err(e) => {
                println!("[plugin_wifi] Scan attempt {} failed: {:?}", attempt, e);
                
                if attempt < 3 {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }
    
    println!("[plugin_wifi] All scan attempts failed");
    unsafe {
        *out_count = 0;
    }
    ptr::null_mut()
}

/// Processes scan results from tokio-wifiscanner into NetworkInfo structures
fn process_scan_results(networks: Vec<Wifi>) -> Vec<NetworkInfo> {
    // Track networks by SSID to handle duplicates
    let mut results: Vec<NetworkInfo> = Vec::with_capacity(networks.len());
    let mut seen_ssids = std::collections::HashSet::new();
    
    for (i, network) in networks.iter().enumerate() {
        // Get SSID as a string
        let ssid_string = if network.ssid.is_empty() {
            format!("Hidden Network {}", i+1)
        } else {
            network.ssid.clone()
        };
        
        // For duplicate SSIDs, use the one with stronger signal
        if seen_ssids.contains(&ssid_string) {
            // Find if we already have this network with worse signal
            if let Some(pos) = results.iter().position(|n| {
                let existing_ssid = unsafe { CStr::from_ptr(n.ssid) }.to_string_lossy();
                
                // Try to parse signal level from string
                let signal_strength = parse_signal_level(&network.signal_level);
                
                existing_ssid == ssid_string && n.signal < signal_strength
            }) {
                // Replace the weaker network
                unsafe { 
                    // Fix: Properly drop the CStrings
                    let _ = CString::from_raw(results[pos].ssid as *mut c_char);
                    let _ = CString::from_raw(results[pos].bssid as *mut c_char);
                    let _ = CString::from_raw(results[pos].security as *mut c_char);
                }
                results.remove(pos);
            } else {
                // Skip this duplicate if it's weaker
                continue;
            }
        }
        seen_ssids.insert(ssid_string.clone());
        
        // Create BSSID string - it's just a string, not an Option
        let bssid_string = if network.mac.is_empty() {
            format!("Unknown-{}", i+1)
        } else {
            network.mac.clone()
        };
        
        // Map security type (this is a string field)
        let security_string = security_type_to_string(&network.security);
        
        // Get channel by parsing the string
        let channel = parse_channel(&network.channel);
        
        // Calculate frequency from channel
        let frequency = channel_to_frequency(channel);
        
        // Parse signal level from string
        let signal = parse_signal_level(&network.signal_level);
        
        // Log the network details
        println!(
            "[plugin_wifi] Network {}: SSID: {}, BSSID: {}, Signal: {}, Channel: {}, Security: {}, Frequency: {}MHz",
            i, ssid_string, bssid_string, signal, channel, security_string, frequency
        );
        
        // Create CStrings for FFI
        let ssid_cstr = CString::new(ssid_string).unwrap_or_default().into_raw();
        let bssid_cstr = CString::new(bssid_string).unwrap_or_default().into_raw();
        let security_cstr = CString::new(security_string).unwrap_or_default().into_raw();
        
        // Create NetworkInfo struct
        let network_info = NetworkInfo {
            ssid: ssid_cstr,
            bssid: bssid_cstr,
            signal,
            channel: channel as i32,
            security: security_cstr,
            frequency,
        };
        
        results.push(network_info);
    }
    
    results
}

/// Parse channel from string
fn parse_channel(channel_str: &str) -> u32 {
    channel_str.parse::<u32>().unwrap_or(0)
}

/// Parse signal level from string
fn parse_signal_level(signal_str: &str) -> i32 {
    signal_str.parse::<i32>().unwrap_or(0)
}

/// Connects to a WiFi network using the wifi-rs crate
pub fn connect_wifi(ssid: &str, password: &str) -> bool {
    println!("[plugin_wifi] Attempting to connect to {} using wifi-rs", ssid);
    
    // Create a new WiFi instance with None config
    // Fixed initialization without the unnecessary match
    let mut wifi = wifi_rs::WiFi::new(None);
    
    for attempt in 1..=3 {
        println!("[plugin_wifi] Connection attempt {} of 3", attempt);
        
        // Use the Connectivity trait explicitly
        match <wifi_rs::WiFi as Connectivity>::connect(&mut wifi, ssid, password) {
            Ok(success) => {
                if success {
                    println!("[plugin_wifi] Successfully connected to {}", ssid);
                    return true;
                } else {
                    println!("[plugin_wifi] Connect call succeeded but reported failure");
                }
            },
            Err(e) => {
                println!("[plugin_wifi] Connection attempt {} failed: {:?}", attempt, e);
            }
        }
        
        if attempt < 3 {
            println!("[plugin_wifi] Waiting before next connection attempt...");
            thread::sleep(Duration::from_secs(2));
        }
    }
    
    println!("[plugin_wifi] All connection attempts failed");
    false
}

/// Legacy compatibility function for older code
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub fn connect_wifi_impl(ssid: &str, password: &str) -> bool {
    connect_wifi(ssid, password)
}

/// Legacy compatibility function for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn connect_wifi_impl(_: &str, _: &str) -> bool {
    println!("[plugin_wifi] WiFi connection not available on this platform");
    false
}


