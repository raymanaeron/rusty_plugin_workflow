# How to use the logger in the plugin

This document explains how to use the logging functionality provided by the plugin_core crate.

## Setup

### 1. Update your Plugin's Cargo.toml

Enable the logging feature in your plugin's `Cargo.toml`:

```toml
[dependencies]
plugin_core = { path = "../../plugin_core", features = ["logging"] }
```

### 2. Initialize in your Plugin Code

Add the logger initialization to your plugin's `on_load` function:

```rust
#[ctor::ctor]
fn on_load() {
    // Initialize the logger for this plugin
    if let Err(e) = plugin_core::init_logger("plugin_myfeature") {
        eprintln!("[plugin_myfeature] Failed to initialize logger: {}", e);
    }
    
    println!("[plugin_myfeature] >>> LOADED");
}
```

### 3. Import Macros and Use Logging

For basic usage with the default macros:

```rust
use plugin_core::{log_debug, log_info, log_warn, log_error};

fn some_function() {
    log_info!("Function started");
    
    if let Err(e) = operation() {
        log_error!("Operation failed: {}", e);
        return;
    }
    
    log_debug!("Debug details: {:?}", some_data);
}
```

For advanced logging features (timing, error handling, etc.):

```rust
// Add this at the top of your file
plugin_core::initialize_logger_attributes!();

// Then use attribute macros
#[plugin_core::log_entry_exit]
fn process_data(user_id: &str) {
    // Function implementation
    // This will automatically log ENTRY and EXIT
}

#[plugin_core::log_errors]
fn risky_operation() -> Result<(), String> {
    // Function implementation
    // Errors will be automatically logged
}

#[plugin_core::measure_time]
fn performance_critical_task() {
    // Function implementation
    // Execution time will be logged
}
```

## Initialization Order

The SDK follows this initialization order:

1. The engine starts and initializes its logger instance
2. Plugin libraries are loaded
3. Each plugin's `on_load` function is called (where you should initialize your logger)
4. The engine calls the plugin's `run` function

This means your plugin doesn't need to create an entirely new logger instance - it simply connects to the existing logger configuration used by the engine.

## Available Log Levels

- `log_debug!()` - Detailed information for debugging
- `log_info!()` - General information about normal operation
- `log_warn!()` - Warning situations that might need attention
- `log_error!()` - Error conditions that should be addressed

## Context Parameter

All logging macros accept an optional context parameter:

```rust
// Basic logging
log_info!("User profile updated");

// With context
let user_id = "12345";
log_info!("User profile updated", Some(format!("user_id={}", user_id)));
```

## Configuration

The logger is configured via the `app_config.toml` file at the application root. Your plugin doesn't need to provide configuration - it uses the engine's configuration.

## Fallback Behavior

If the logging system can't be initialized properly, your logs will fall back to console output to ensure messages are never lost.

## Runtime Feature Detection

You can check if logging is enabled at runtime:

```rust
if plugin_core::logging_enabled() {
    // Do something that only makes sense when logging is enabled
}
```
