fn main() {
    // No custom build steps needed since we're using pure Rust crates now
    println!("cargo:rerun-if-changed=build.rs");
}
