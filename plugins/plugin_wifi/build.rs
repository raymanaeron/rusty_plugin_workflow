fn main() {
    if cfg!(target_os = "macos") {
        cc::Build::new()
            .file("src/wifi_corewlan.m")
            .flag("-fobjc-arc")
            .flag("-mmacosx-version-min=11.0") // Set deployment target
            .compile("wifi_corewlan");

        println!("cargo:rustc-link-lib=framework=CoreWLAN");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=objc");
    } else {
        println!("cargo:warning=Skipping CoreWLAN build as it is not macOS.");
    }
}
