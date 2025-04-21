# OOBE Plugin Architecture

## Background and Motivation

Out-of-Box Experience (OOBE) is traditionally treated as a fixed part of firmware, tightly coupled with the system image. Any change—whether it is a regulatory update, a localization fix, or a new onboarding step—requires rebuilding and revalidating the firmware image. This rigidity impedes innovation and slows down the ability to respond to customer needs.

The OOBE plugin engine addresses this constraint by shifting setup logic from firmware to a runtime-executed, plugin-based model. Each step in the onboarding process is encapsulated in a dynamically loadable plugin that the engine discovers and executes according to a declarative plan. This approach enables updates without firmware rebuilds, modular development, dynamic workflow composition, and runtime observability.

## Plugin Model and Interface

Each plugin is built as a dynamic library implementing a fixed ABI surface defined by the engine. The core entry point is `create_plugin()`, which returns a pointer to a `Plugin` struct:

```rust
#[repr(C)]
pub struct Plugin {
    pub name: extern "C" fn() -> *const c_char,
    pub plugin_route: extern "C" fn() -> *const c_char,
    pub run: extern "C" fn(ctx: *const PluginContext),
    pub get_static_content_path: extern "C" fn() -> *const c_char,
    pub get_api_resources: extern "C" fn(out_len: *mut usize) -> *const Resource,
    pub handle_request: extern "C" fn(request: *const ApiRequest) -> *mut ApiResponse,
    pub cleanup: extern "C" fn(response: *mut ApiResponse),
    pub run_workflow: Option<extern "C" fn(input: *const ApiRequest) -> *mut ApiResponse>,
    pub on_progress: Option<extern "C" fn() -> *mut ApiResponse>,
    pub on_complete: Option<extern "C" fn() -> *mut ApiResponse>,
}
```

This ABI enables the engine to load plugins without relying on Rust-specific constructs like traits or vtables, preserving platform independence and safety across FFI boundaries.

## Engine Initialization and Plugin Lifecycle

At startup, the engine reads a static configuration for required plugins (e.g., WiFi, Terms, Status), followed by an optional dynamic execution plan fetched from disk or a remote location. For each plugin:

1. The engine uses `libloading::Library` to load the `.so`/`.dll`.
2. It calls `create_plugin()` and extracts metadata, including `plugin_route`, `name`, and static asset path.
3. It stores the loaded plugin in a registry mapped by both name and route.
4. It invokes the plugin’s `run()` method with a `PluginContext` containing configuration parameters.
5. It retains the library handle to prevent unloading during execution.

## Execution Plan and Plugin Metadata

Plugins listed in the execution plan are described using `PluginMetadata`, which includes fields like:

```toml
[[plugins]]
name = "plugin_wifi"
plugin_route = "wifi"
version = "1.0.0"
plugin_location_type = "local"
plugin_base_path = "./plugins"
run_async = true
visible_in_ui = true
```

Each plugin is loaded from `plugin_base_path` and initialized in the same way as static plugins. Conditional loading, version pinning, or remote fetching (e.g., via S3) can be implemented with no change to engine logic.

## Routing Architecture

The engine exposes two major routing layers using Axum:

### Static Web Content

Each plugin defines a static content path like `wifi/web` via `get_static_content_path()`. This path is mounted at:

```
/wifi/web/*
```

So the file `wifi/web/step-wifi.html` is accessible at `/wifi/web/step-wifi.html`.

### REST APIs

REST APIs are declared by the plugin in `get_api_resources()`. These return a slice of `Resource`, each describing a relative path and supported HTTP methods. The engine maps these under:

```
/api/<plugin_route>/<resource>
```

For example, `plugin_route = "wifi"` and `resource = "network"` results in:

- `GET /api/wifi/network` → scan
- `POST /api/wifi/network` → connect

## Web UI Shell and Frontend Composition

The frontend is a minimal HTML shell hosted from `/webapp/index.html`. It contains a header and a `<main id="content">` placeholder. At load time, `app.js` dispatches control to a router module which dynamically injects plugin UIs based on the current URL.

```js
routeTo("/wifi/web");

function routeTo(path) {
  const pluginName = path.split("/")[1];
  const htmlUrl = `/${pluginName}/web/step-${pluginName}.html`;
  const jsUrl = `/${pluginName}/web/step-${pluginName}.js`;

  // Fetch HTML and activate plugin
}
```

Each plugin must export an `activate(container)` function in its JS module. This function binds DOM events to the plugin's REST APIs and handles UI updates.

## Progress Polling and Task Chaining

For long-running or multi-step workflows, the engine supports progress callbacks via `on_progress()` and `on_complete()`.

- `on_progress()` returns a `ApiResponse` with structured status.
- `on_complete()` returns HTTP 200 on task success.

The engine polls these methods every second using `tokio::spawn`. If progress messages are returned, they are forwarded as `POST` requests to the `status` plugin via `handle_request()`. This allows one plugin to report updates to another without a shared memory bus.

When `on_complete()` indicates success, the engine can invoke `run_workflow()` on the next plugin, allowing chained execution.

## Logger System and HTTP Telemetry

The engine includes a flexible, modular logging system based on runtime configuration:

- The logger is initialized from a TOML config file (`app_config.toml`)
- It supports both file-based and HTTP-based destinations.
- Logger behavior is driven by the following config section:

```toml
[logging]
type = "http"
http_endpoint = "http://localhost:9000/logs"
threshold = "debug"
```

Logging follows the `Logger` trait defined in `log_contracts.rs`, and is implemented via `LogWriter`. Messages are serialized to `LogEntry` and dispatched to the configured destination.

If `type = "file"`, logs are written in JSON lines to a rotating log file:

```rust
FileLogDestination::new(PathBuf::from("app.log"), 10 * 1024 * 1024);
```

If `type = "http"`, logs are posted to the endpoint as structured JSON payloads:

```rust
HttpLogDestination::new("http://example.com/logs");
```

Structured log context (MDC) is supported using `LogWriter::set_context("session=abc123")`.

The logger is accessed globally via:

```rust
LoggerLoader::get_logger().info("Plugin wifi started");
```

## Conclusion

This architecture cleanly separates orchestration from execution. Each plugin encapsulates a feature, exposes a REST interface and optional UI, and operates in isolation. The engine acts as a thin orchestrator—loading plugins, executing plans, routing requests, and coordinating flow.

By moving onboarding logic out of firmware and into runtime-executed plugins, the system gains agility, modularity, and real-time introspection. Updates are faster, plugins are reusable, and setup flows can adapt dynamically based on device, customer, or context.

With full support for static content, REST APIs, progress polling, chained workflows, and HTTP-based logging, this architecture lays a resilient and extensible foundation for onboarding across diverse devices and product lines.