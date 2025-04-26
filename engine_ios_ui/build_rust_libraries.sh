#!/bin/bash
# filepath: c:\Code\rusty_plugin_workflow\engine_ios_ui\build_rust_libraries.sh

set -e

# Ensure the necessary tools are installed
if ! command -v rustup &> /dev/null; then
    echo "Error: rustup not found. Please install Rust first."
    exit 1
fi

# Add iOS targets
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios

# Build directory
mkdir -p target/ios

# Path to the Rust library project
RUST_PROJECT_PATH="../engine"

echo "Building Rust libraries for iOS..."

# Build for device (arm64)
echo "Building for iOS devices (arm64)..."
cargo build --manifest-path "$RUST_PROJECT_PATH/Cargo.toml" --target aarch64-apple-ios --release

# Build for simulator (arm64 and x86_64)
echo "Building for iOS simulator (arm64)..."
cargo build --manifest-path "$RUST_PROJECT_PATH/Cargo.toml" --target aarch64-apple-ios-sim --release

echo "Building for iOS simulator (x86_64)..."
cargo build --manifest-path "$RUST_PROJECT_PATH/Cargo.toml" --target x86_64-apple-ios --release

# Create universal library for simulator
echo "Creating universal library for simulator..."
lipo -create \
    "$RUST_PROJECT_PATH/target/aarch64-apple-ios-sim/release/libengine.a" \
    "$RUST_PROJECT_PATH/target/x86_64-apple-ios/release/libengine.a" \
    -output "target/ios/libengine_simulator.a"

# Copy device library
cp "$RUST_PROJECT_PATH/target/aarch64-apple-ios/release/libengine.a" "target/ios/libengine_device.a"

echo "Done building Rust libraries for iOS"
echo "Libraries are available at:"
echo "  - target/ios/libengine_simulator.a (for simulator)"
echo "  - target/ios/libengine_device.a (for devices)"

echo
echo "Add these libraries to your Xcode project and configure build settings:"
echo "1. In Xcode, add the libraries to your project"
echo "2. Add '-lc++' to 'Other Linker Flags'"
echo "3. Add path to libraries in 'Library Search Paths'"
echo "4. Add path to header files in 'Header Search Paths'"