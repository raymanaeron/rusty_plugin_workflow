# Rust Engine iOS UI

This is an iOS app that wraps the Rust engine server, displaying its UI in a WebView.

## Prerequisites

- Xcode 14.0 or newer
- Rust with iOS targets (see setup instructions below)
- Cargo and Rustup

## Setup Rust for iOS Development

1. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Add iOS targets:
   ```bash
   rustup target add aarch64-apple-ios
   rustup target add aarch64-apple-ios-sim
   rustup target add x86_64-apple-ios
   ```

## Building the Rust Libraries

1. Run the build script:
   ```bash
   cd engine_ios_ui
   chmod +x build_rust_libraries.sh
   ./build_rust_libraries.sh
   ```

2. The script will create two libraries:
   - `target/ios/libengine_simulator.a` - Universal library for iOS simulators
   - `target/ios/libengine_device.a` - Library for physical iOS devices

## Setting Up the Xcode Project

1. Open the Xcode project: `EngineIOSUI.xcodeproj`

2. Add the Rust libraries to your project:
   - Drag and drop the appropriate library file (`libengine_simulator.a` or `libengine_device.a`) into your project
   - Make sure "Copy items if needed" is checked
   - Add to your app's target

3. Configure build settings:
   - In your target's Build Settings, find "Library Search Paths" and add the path to the directory containing the Rust library
   - Add "-lc++" to "Other Linker Flags"

4. If you have header files to include:
   - Add the path to the headers in "Header Search Paths"

## Running the App

1. Select your target device/simulator in Xcode
2. Build and run the project (⌘R)

## Debugging

If you encounter issues:

1. **Server not starting**:
   - Check the Xcode console for error messages
   - Verify the FFI functions are correctly implemented in the Rust library
   - Make sure the library is properly linked

2. **WebView not connecting**:
   - Verify network permissions in Info.plist
   - Check if server is actually running on ports 8080/8081
   - On real devices, ensure the app has local network permissions

3. **WebSockets not working**:
   - Check browser console in WebView for connection errors
   - Verify the WebView configuration allows WebSocket connections

## App Architecture

- `EngineIOSUIApp.swift` - Main app entry point that starts the Rust server
- `RustServerController.swift` - Manages server lifecycle and status
- `ContentView.swift` - Main UI with loading indicator and WebView
- `WebViewContainer.swift` - WKWebView wrapper configured for local connections
- `RustFFI.swift` - FFI bridge to Rust functions
