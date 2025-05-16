#!/bin/bash
# clean_ios_builds.sh - Script to clean iOS build artifacts

set -e  # Exit immediately on any error

echo "iOS Build Cleanup Utility"
echo "========================="

# Define cleanup options
CLEAN_ENGINE_TARGETS=false
CLEAN_BUILD_DIR=false
CLEAN_TARGET_DIR=false
CLEAN_ALL=false

# Parse command-line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --engine) CLEAN_ENGINE_TARGETS=true ;;
        --build) CLEAN_BUILD_DIR=true ;;
        --target) CLEAN_TARGET_DIR=true ;;
        --all) CLEAN_ALL=true ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
    shift
done

# If no args provided, show help
if [[ "$CLEAN_ENGINE_TARGETS" == "false" && "$CLEAN_BUILD_DIR" == "false" && "$CLEAN_TARGET_DIR" == "false" && "$CLEAN_ALL" == "false" ]]; then
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --engine   Clean iOS-specific Rust build targets"
    echo "  --build    Clean engine_ios_ui/build directory"
    echo "  --target   Clean target/ios directory"
    echo "  --all      Clean all iOS build artifacts"
    exit 0
fi

# Set all flags to true when --all is specified
if [[ "$CLEAN_ALL" == "true" ]]; then
    CLEAN_ENGINE_TARGETS=true
    CLEAN_BUILD_DIR=true
    CLEAN_TARGET_DIR=true
fi

# Clean engine targets
if [[ "$CLEAN_ENGINE_TARGETS" == "true" ]]; then
    echo "Cleaning iOS Rust build targets..."
    cargo clean --target aarch64-apple-ios
    cargo clean --target aarch64-apple-ios-sim
    cargo clean --target x86_64-apple-ios
    echo "iOS Rust build targets cleaned."
fi

# Clean build directory
if [[ "$CLEAN_BUILD_DIR" == "true" ]]; then
    echo "Cleaning engine_ios_ui/build directory..."
    # Remove all files but keep .gitkeep
    find engine_ios_ui/build -type f -not -name ".gitkeep" -delete
    echo "engine_ios_ui/build directory cleaned (kept directory structure)."
fi

# Clean target/ios directory
if [[ "$CLEAN_TARGET_DIR" == "true" ]]; then
    echo "Cleaning target/ios directory..."
    rm -rf target/ios
    echo "target/ios directory removed."
fi

echo "Cleanup completed!"
