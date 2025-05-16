# iOS Integration Guide for Rusty Plugin Workflow

This guide outlines how to integrate the Rust libraries into your iOS Xcode project, specifically handling the differences between device and simulator builds.

## 1. Prerequisites

- Xcode 14.0 or higher
- iOS 14.0 or higher (target deployment)
- Built Rust libraries (run `./b_ios.sh --release` in the project root first)

## 2. Library Structure

The build process creates separate libraries for device and simulator due to architecture overlap:

- **Device library**: `engine_ios_ui/build/device/librusty_plugin.a`
  - Architecture: arm64 (for physical iOS devices)
  
- **Simulator library**: `engine_ios_ui/build/simulator/librusty_plugin.a`
  - Architectures: arm64 and x86_64 (for simulator on Intel and Apple Silicon Macs)

- **Header files**: `engine_ios_ui/build/include/*.h`

## 3. Integration Steps in Xcode

### Step 1: Add libraries to your project

1. In Xcode, right-click your project in the Project Navigator
2. Select "Add Files to [Your Project]..."
3. Navigate to and select both libraries:
   - `build/device/librusty_plugin.a`
   - `build/simulator/librusty_plugin.a`
4. In the options dialog, ensure:
   - [x] "Copy items if needed" is unchecked
   - [x] "Create groups" is selected
   - [x] Your app target is checked
5. Click "Add"

### Step 2: Configure Build Settings

1. Select your project in the Project Navigator
2. Select your app target
3. Go to "Build Settings" tab
4. Make the following changes:
   
   a. **Library Search Paths** (LIBRARY_SEARCH_PATHS):
   ```
   // For Debug configuration
   $(PROJECT_DIR)/build/$(PLATFORM_NAME)
   
   // Where PLATFORM_NAME will be:
   // - "simulator" for Simulator builds
   // - "device" for Device builds
   ```
   
   b. **Header Search Paths** (HEADER_SEARCH_PATHS):
   ```
   $(PROJECT_DIR)/build/include
   ```
   
   c. **Other Linker Flags** (OTHER_LDFLAGS):
   ```
   -lc++ -lrusty_plugin
   ```

### Step 3: Add Swift Wrapper

Create a Swift file to interface with the Rust code:

```swift
import Foundation

// Import the C interface functions from the Rust library
@_cdecl("rust_start_server")
public func rust_start_server() -> Int32

@_cdecl("rust_stop_server")
public func rust_stop_server() -> Int32

// Add wrapper class
class RustServer {
    static func start() -> Bool {
        return rust_start_server() == 0
    }
    
    static func stop() -> Bool {
        return rust_stop_server() == 0
    }
}
```

### Step 4: Create Bridging Header (if needed)

If your project uses Objective-C code that needs to interact with the Rust library:

1. Create a file named `YourProject-Bridging-Header.h`
2. Add:
   ```objc
   #import "engine.h"
   ```
3. Set the file in your target's build settings under "Swift Compiler - General" > "Objective-C Bridging Header"

## 4. Troubleshooting

### Architecture Issues

If you see errors like "file was built for archive which is not the architecture being linked":

1. Ensure you're using the correct library for the current build target
2. Check that the `LIBRARY_SEARCH_PATHS` are correctly set for each platform
3. Clean the build folder (Product > Clean Build Folder) and rebuild

### Linker Errors

For "symbol not found" errors:

1. Verify the function exists in the Rust library
2. Check that the header matches the Rust implementation
3. Ensure `-lrusty_plugin` is included in "Other Linker Flags"

### Build Script Issues

If the build script fails:

1. Make sure Xcode command line tools are installed: `xcode-select --install`
2. Verify Rust targets are installed: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios`
3. Check for error messages in the build output

## 5. Running the Demo App

The `engine_ios_ui` directory contains a sample Xcode project that demonstrates the integration:

1. Navigate to the `engine_ios_ui` directory
2. Run `./setup_xcode_project.sh` to prepare the project
3. Open `EngineIOSUI.xcodeproj` in Xcode
4. Select a simulator or connected device
5. Build and run

## 6. Additional Resources

- Sample Xcode project: `/engine_ios_ui/EngineIOSUI.xcodeproj`
- Build script documentation: `b_ios.sh --help`
- API documentation in header files: `/build/include/*.h`
