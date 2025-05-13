# Android WebView Implementation Guide

This document outlines the steps required to create an Android application that will display our engine's web content running on localhost:8080, similar to how our desktop UI works.

## Overview

The Android implementation will follow the same general approach as our desktop UI:
1. Start the Rust engine in a background thread
2. Wait for the server to become available
3. Display the web content from localhost:8080 in an Android WebView

## Step-by-Step Implementation

### 1. Project Setup

1. Create a new Android project in Android Studio
   - Target Android API level 21+ (Android 5.0 Lollipop and higher)
   - Use Kotlin as the primary language
   - Create a basic empty activity project

### 2. Add Required Dependencies

Add the following to your `build.gradle` (app module):
- NDK support for native code
- Rust library integration dependencies
- WebView related dependencies

### 3. Prepare Rust Engine for Android

1. Set up cargo-ndk in your development environment
2. Configure Rust build targets for Android architectures:
   - arm64-v8a (aarch64-linux-android)
   - armeabi-v7a (armv7-linux-androideabi)
   - x86 (i686-linux-android)
   - x86_64 (x86_64-linux-android)

3. Create JNI bindings for your Rust engine:
   - Create a `rustlib` module in your Android project
   - Configure build scripts to compile Rust code for Android targets

### 4. Main Application Components

1. **MainActivity.kt**: Entry point for your Android app
2. **EngineService.kt**: Background service to run the Rust engine
3. **WebViewActivity.kt**: Activity containing the WebView to display localhost content

### 5. Android Permissions

Add these to your `AndroidManifest.xml`:
- `android.permission.INTERNET`
- `android.permission.FOREGROUND_SERVICE`

### 6. Start Engine in Background Service

1. Create an Android Service that:
   - Loads the native Rust library via JNI
   - Starts the engine on a background thread
   - Monitors engine status
   - Provides binding for activities to connect

### 7. WebView Setup

1. Configure WebView in your activity:
   - Enable JavaScript
   - Allow local content
   - Configure WebView client to handle navigation
   - Set up debugging if needed

2. Special WebView configurations:
   - Override WebViewClient to handle localhost URLs
   - Configure WebChromeClient for full feature support
   - Set up any required JavaScript interfaces

### 8. Port Management

1. Handle localhost binding on Android:
   - Android requires special handling for localhost
   - Consider using WebSocket connections instead of direct HTTP
   - Or use proper loopback address (10.0.2.2 for emulator)

2. Port forwarding configuration:
   - Set up network security configuration
   - Handle SSL/TLS certificates if needed

### 9. Communication Between Components

1. Implement communication between:
   - Native Rust code and Java/Kotlin (using JNI)
   - Service (running engine) and Activity (displaying WebView)
   - WebView JavaScript and Android code (using JavascriptInterface)

### 10. Resource and Lifecycle Management

1. Handle application lifecycle events:
   - Pause/resume behavior
   - Engine shutdown on app close
   - Memory management concerns

2. Resource cleanup:
   - Proper shutdown of the Rust engine
   - WebView resource cleanup
   - Service termination

### 11. Testing Considerations

1. Test on physical devices and emulators
2. Verify WebView compatibility across Android versions
3. Test network connectivity and localhost access

## Implementation Notes

- Android's security model differs from desktop, requiring careful handling of local networking
- Consider using a higher initial port number (>8080) to avoid potential conflicts
- WebView implementations vary by Android version; test thoroughly

## Next Steps

1. Set up the basic Android project structure
2. Configure Rust cross-compilation for Android targets
3. Implement the background service for the engine
4. Create the WebView activity and configure it properly
5. Test on different Android versions and devices