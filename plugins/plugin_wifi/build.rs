use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    if target_os == "macos" {
        // Compile the Objective-C code
        cc::Build::new()
            .file("src/wifi_corewlan.m")
            .flag("-framework")
            .flag("CoreWLAN")
            .flag("-framework")
            .flag("CoreLocation")
            .flag("-framework")
            .flag("Foundation")   // Add Foundation framework
            .compile("wifi_corewlan");
            
        // Copy the .plist files to the output directory
        let out_dir = env::var("OUT_DIR").unwrap();
        
        // Check if the plist files exist before trying to copy
        if Path::new("src/Info.plist").exists() {
            let dest_path = Path::new(&out_dir).join("Info.plist");
            fs::copy("src/Info.plist", &dest_path).expect("Failed to copy Info.plist");
        } else {
            println!("cargo:warning=Info.plist not found, skipping copy");
        }
        
        if Path::new("src/entitlements.plist").exists() {
            let entitlements_path = Path::new(&out_dir).join("entitlements.plist");
            fs::copy("src/entitlements.plist", &entitlements_path).expect("Failed to copy entitlements.plist");
        } else {
            println!("cargo:warning=entitlements.plist not found, skipping copy");
        }
        
        // Link against the required frameworks
        println!("cargo:rustc-link-lib=framework=CoreWLAN");
        println!("cargo:rustc-link-lib=framework=CoreLocation");
        println!("cargo:rustc-link-lib=framework=Foundation");  // Add Foundation framework
        
        // Set Mac OS target version to match the build environment
        println!("cargo:rustc-link-arg=-mmacosx-version-min=10.13");
    }
}
