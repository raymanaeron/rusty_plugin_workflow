fn main() {
    // Tell Tauri where the config file is
    println!("cargo:rerun-if-changed=tauri.conf.json");
}
