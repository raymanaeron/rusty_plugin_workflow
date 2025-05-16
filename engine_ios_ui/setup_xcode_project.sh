#!/bin/bash
# setup_xcode_project.sh - Helper script to set up the Xcode project

set -e  # Exit immediately on any error

echo "Setting up Xcode project for Rusty Plugin Workflow"
echo "=================================================="

# Check if Xcode is installed
if ! xcode-select -p &> /dev/null; then
    echo "Error: Xcode not found. Please install Xcode first."
    exit 1
fi

# Build the Rust libraries first if they don't exist
if [ ! -f "build/simulator/librusty_plugin.a" ] || [ ! -f "build/device/librusty_plugin.a" ]; then
    echo "Rust libraries not found. Building them first..."
    cd ..
    ./b_ios.sh --release
    cd engine_ios_ui
fi

# Check if the libraries are now available
if [ ! -f "build/simulator/librusty_plugin.a" ] || [ ! -f "build/device/librusty_plugin.a" ]; then
    echo "Error: Failed to build Rust libraries. Please check the build output."
    exit 1
fi

# Ensure the build directories exist with proper structure
mkdir -p build/simulator
mkdir -p build/device
mkdir -p build/include

# Check if header files exist and copy them if needed
if [ ! -f "build/include/engine.h" ]; then
    echo "Copying header files..."
    if [ -d "../engine/include" ]; then
        cp -R ../engine/include/* build/include/
    else
        echo "Warning: No header files found in ../engine/include"
    fi
fi

# Open Xcode project
echo "Opening Xcode project..."
open EngineIOSUI.xcodeproj

echo ""
echo "Setup complete! Important build settings for Xcode:"
echo "1. Library Search Paths:"
echo "   - For simulator: \$(PROJECT_DIR)/build/simulator"
echo "   - For device: \$(PROJECT_DIR)/build/device"
echo "   You can set this conditionally with: \$(PROJECT_DIR)/build/\$(PLATFORM_NAME)"
echo ""
echo "2. Header Search Paths:"
echo "   - \$(PROJECT_DIR)/build/include"
echo ""
echo "3. Other Linker Flags:"
echo "   - -lc++ -lrusty_plugin"
echo ""
echo "4. For Build Rules - add a custom rule to use the appropriate library based on the target"
echo ""
echo "5. Make sure to select appropriate team and signing settings in the Signing & Capabilities tab"
echo ""
