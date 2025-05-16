#!/bin/bash
set -e  # Exit immediately on any error

# === Parse arguments ===
MODE="debug"
CARGO_FLAG=""
if [[ "$1" == "--release" ]]; then
  MODE="release"
  CARGO_FLAG="--release"
fi
TARGET="target/$MODE"
TARGET_DIR="$(pwd)/target"  # Absolute path to target directory

echo ">> TARGET is: $TARGET"
echo "[BUILD] Mode set to $MODE"
echo "[BUILD] Output path: $TARGET"

# === Check for Xcode and iOS SDK ===
echo "Checking for Xcode command line tools..."
if ! xcode-select -p &> /dev/null; then
    echo "Error: Xcode command line tools not found."
    echo "Install them with: xcode-select --install"
    exit 1
fi

# Check for iOS SDK with detailed diagnostics
echo "Checking for iOS SDK..."
SDK_PATH=$(xcrun --sdk iphoneos --show-sdk-path 2>/dev/null)
SDK_STATUS=$?

if [ $SDK_STATUS -ne 0 ]; then
    echo "Warning: iOS SDK detection failed with status $SDK_STATUS"
    echo "Current Xcode path: $(xcode-select -p)"
    echo "Attempting to explicitly set Xcode path..."
    
    # Try to explicitly set Xcode path
    if [ -d "/Applications/Xcode.app/Contents/Developer" ]; then
        echo "Setting Xcode path to /Applications/Xcode.app/Contents/Developer"
        sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
        
        # Try again with the new path
        SDK_PATH=$(xcrun --sdk iphoneos --show-sdk-path 2>/dev/null)
        SDK_STATUS=$?
        
        if [ $SDK_STATUS -eq 0 ]; then
            echo "Success! iOS SDK found at: $SDK_PATH"
        else
            echo "Still unable to locate iOS SDK despite setting Xcode path."
        fi
    else
        echo "Xcode.app not found in /Applications folder."
    fi
    
    # If we still can't find it, check for license agreement
    if [ $SDK_STATUS -ne 0 ]; then
        echo "Checking if license agreement needs to be accepted..."
        
        # Try to accept license
        sudo xcodebuild -license accept 2>/dev/null
        
        # Try again after license acceptance
        SDK_PATH=$(xcrun --sdk iphoneos --show-sdk-path 2>/dev/null)
        SDK_STATUS=$?
        
        if [ $SDK_STATUS -eq 0 ]; then
            echo "Success! iOS SDK found at: $SDK_PATH after accepting license."
        else
            echo "Error: Unable to locate iOS SDK."
            echo "Please ensure Xcode is properly installed and set up."
            exit 1
        fi
    fi
fi

echo "Using iOS SDK at: $SDK_PATH"

# === Add iOS targets to Rust ===
echo "Adding iOS build targets to Rust..."
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios

# === Create iOS build directories ===
echo "Creating iOS build directories..."
mkdir -p "$TARGET_DIR/ios"
mkdir -p "engine_ios_ui/build/include"

# === Build all plugins and engine components ===
echo "Building all plugins and engine components..."

# Build Engine
echo "Building engine for iOS devices (arm64)..."
cargo build --manifest-path engine/Cargo.toml --target aarch64-apple-ios $CARGO_FLAG

echo "Building engine for iOS simulator (arm64)..."
cargo build --manifest-path engine/Cargo.toml --target aarch64-apple-ios-sim $CARGO_FLAG

echo "Building engine for iOS simulator (x86_64)..."
cargo build --manifest-path engine/Cargo.toml --target x86_64-apple-ios $CARGO_FLAG

# Build Plugins (similar to b_mac.sh but for iOS targets)
echo "Building plugins for iOS..."

# Function to build a plugin for all iOS targets
build_plugin() {
    local plugin_path=$1
    local plugin_name=$(basename "$plugin_path")
    echo "Building $plugin_name for iOS..."
    
    cargo build --manifest-path "$plugin_path/Cargo.toml" --target aarch64-apple-ios $CARGO_FLAG
    cargo build --manifest-path "$plugin_path/Cargo.toml" --target aarch64-apple-ios-sim $CARGO_FLAG
    cargo build --manifest-path "$plugin_path/Cargo.toml" --target x86_64-apple-ios $CARGO_FLAG
}

# Build all plugins for iOS
build_plugin "plugins/plugin_welcome"
build_plugin "plugins/plugin_wifi"
build_plugin "plugins/plugin_execplan"
build_plugin "plugins/plugin_login"
build_plugin "plugins/plugin_provisioning"
build_plugin "plugins/plugin_terms"
build_plugin "plugins/plugin_settings"
build_plugin "plugins/plugin_status"
build_plugin "plugins/plugin_howto"
build_plugin "plugins/plugin_tutorial"
build_plugin "plugins/plugin_finish"
build_plugin "plugins/plugin_task_agent_headless"

# === Locate built library files ===
echo "Locating built engine library files..."
SIM_ARM64_PATH=""
SIM_X86_64_PATH=""
DEVICE_PATH=""

# Try to find simulator ARM64 build
if [ -f "$TARGET_DIR/aarch64-apple-ios-sim/$MODE/libengine.a" ]; then
    SIM_ARM64_PATH="$TARGET_DIR/aarch64-apple-ios-sim/$MODE/libengine.a"
fi

# Try to find simulator x86_64 build
if [ -f "$TARGET_DIR/x86_64-apple-ios/$MODE/libengine.a" ]; then
    SIM_X86_64_PATH="$TARGET_DIR/x86_64-apple-ios/$MODE/libengine.a"
fi

# Try to find device build
if [ -f "$TARGET_DIR/aarch64-apple-ios/$MODE/libengine.a" ]; then
    DEVICE_PATH="$TARGET_DIR/aarch64-apple-ios/$MODE/libengine.a"
fi

# Check if we found all libraries
if [ -z "$SIM_ARM64_PATH" ] || [ -z "$SIM_X86_64_PATH" ] || [ -z "$DEVICE_PATH" ]; then
    echo "Error: Could not find all required engine library files."
    echo "Missing files:"
    [ -z "$SIM_ARM64_PATH" ] && echo "- ARM64 simulator library"
    [ -z "$SIM_X86_64_PATH" ] && echo "- x86_64 simulator library"
    [ -z "$DEVICE_PATH" ] && echo "- Device library"
    echo "Available library files:"
    find "$TARGET_DIR" -name "libengine.a" | sort
    exit 1
fi

# === Create universal libraries ===
echo "Creating universal library for simulator..."
echo "Using simulator libraries:"
echo "  - ARM64: $SIM_ARM64_PATH"
echo "  - x86_64: $SIM_X86_64_PATH"
lipo -create \
    "$SIM_ARM64_PATH" \
    "$SIM_X86_64_PATH" \
    -output "$TARGET_DIR/ios/libengine_simulator.a"

# Copy device library
echo "Using device library: $DEVICE_PATH"
cp "$DEVICE_PATH" "$TARGET_DIR/ios/libengine_device.a"

# Create a combined library for the Xcode project
echo "Creating universal library for rusty_plugin..."
lipo -create \
    "$TARGET_DIR/ios/libengine_simulator.a" \
    "$TARGET_DIR/ios/libengine_device.a" \
    -output "engine_ios_ui/build/librusty_plugin.a"

# === Copy headers ===
echo "Copying headers..."
if [ -d "engine/include" ]; then
    cp -R engine/include/* "engine_ios_ui/build/include/"
fi

# === Create destination folders for web assets (similar to b_mac.sh) ===
echo "Creating plugin destination folders..."
mkdir -p "$TARGET/welcome/web"
mkdir -p "$TARGET/wifi/web"
mkdir -p "$TARGET/execution/web"
mkdir -p "$TARGET/login/web"
mkdir -p "$TARGET/provision/web"
mkdir -p "$TARGET/terms/web"
mkdir -p "$TARGET/settings/web"
mkdir -p "$TARGET/status/web"
mkdir -p "$TARGET/howto/web"
mkdir -p "$TARGET/tutorial/web"
mkdir -p "$TARGET/finish/web"
mkdir -p "$TARGET/taskagent/web"
mkdir -p "$TARGET/webapp"

# === Copy static web assets (similar to b_mac.sh) ===
echo "Copying root web folder to engine output directory..."
rsync -a webapp/ "$TARGET/webapp/"

echo "Copying plugins web folder to engine output directory..."
rsync -a plugins/plugin_welcome/web/ "$TARGET/welcome/web/"
rsync -a plugins/plugin_wifi/web/ "$TARGET/wifi/web/"
rsync -a plugins/plugin_execplan/web/ "$TARGET/execution/web/"
rsync -a plugins/plugin_login/web/ "$TARGET/login/web/"
rsync -a plugins/plugin_provisioning/web/ "$TARGET/provision/web/"
rsync -a plugins/plugin_terms/web/ "$TARGET/terms/web/"
rsync -a plugins/plugin_settings/web/ "$TARGET/settings/web/"
rsync -a plugins/plugin_status/web/ "$TARGET/status/web/"
rsync -a plugins/plugin_howto/web/ "$TARGET/howto/web/"
rsync -a plugins/plugin_tutorial/web/ "$TARGET/tutorial/web/"
rsync -a plugins/plugin_finish/web/ "$TARGET/finish/web/"
rsync -a plugins/plugin_task_agent_headless/web/ "$TARGET/taskagent/web/"

# === Copy config and plan files (similar to b_mac.sh) ===
echo "Copying the app config file to the engine output directory..."
cp app_config.toml "$TARGET/app_config.toml"

echo "Copying the execution_plan.toml file to the engine output directory..."
cp execution_plan.toml "$TARGET/execution_plan.toml"

echo "Copying the external execution_plan.toml file to the engine output directory..."
mkdir -p "$TARGET/ext_plan"
mkdir -p "$TARGET/ext_plan/Echo"
mkdir -p "$TARGET/ext_plan/Echo/1.3"

cp ./ext_plan/Echo/1.3/execution_plan.toml "$TARGET/ext_plan/Echo/1.3/execution_plan.toml"

echo "iOS build completed successfully!"
echo ""
echo "Libraries are available at:"
echo "  - $TARGET_DIR/ios/libengine_simulator.a (for simulator)"
echo "  - $TARGET_DIR/ios/libengine_device.a (for device)"
echo "  - engine_ios_ui/build/librusty_plugin.a (universal library)"
echo ""
echo "To use in your Xcode project:"
echo "1. Add the libraries to your project"
echo "2. Add '-lc++' to 'Other Linker Flags'"
echo "3. Add path to libraries in 'Library Search Paths'"
echo "4. Add path to header files in 'Header Search Paths'"
