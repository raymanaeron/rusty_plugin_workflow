# OOBE SDK Plugin Architecture

## Core Plugin Interface

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

This ABI enables the engine to load plugins without relying on Rust-specific constructs like traits or vtables, preserving platform independence and safety across FFI boundaries. Each function in the interface serves a specific purpose:

- `name` and `plugin_route`: Provide identity and routing information
- `run`: Initializes the plugin with configuration parameters
- `get_static_content_path`: Returns the path to the plugin's web assets
- `get_api_resources`: Defines the REST API endpoints exposed by the plugin
- `handle_request`: Processes incoming API requests
- `cleanup`: Responsible for deallocating memory allocated by the plugin
- `run_workflow`, `on_progress`, `on_complete`: Optional callbacks for long-running workflow tasks

## Plugin Implementation

Plugins implement this interface through the `plugin_core` crate, which provides macros that hide much of the FFI complexity from plugin developers. The most common implementation pattern uses the `declare_plugin!` macro:

```rust
declare_plugin!(
    "plugin_welcome",        // Internal plugin name
    "welcome",               // Routing path
    run,                     // Initialization function
    get_static_content_path, // Static content function
    get_api_resources,       // API resources function
    handle_request,          // Request handler function
    cleanup                  // Memory cleanup function
);
```

For plugins requiring workflow functionality, the macro accepts additional parameters for workflow management:

```rust
declare_plugin!(
    "plugin_name",
    "route_name",
    run, 
    get_static_content_path,
    get_api_resources,
    handle_request,
    cleanup,
    run_workflow,   // Workflow entry point 
    on_progress,    // Progress reporting callback
    on_complete     // Completion callback
);
```

## Resource Definition

Plugins define their REST API endpoints using the `Resource` struct:

```rust
#[repr(C)]
pub struct Resource {
    pub path: *const c_char,
    pub supported_methods: *const HttpMethod,
    _marker: PhantomData<*const ()>,
}
```

The engine mounts these resources at `/api/<plugin_route>/<resource_path>`. The `static_resource` helper function simplifies creating these definitions:

```rust
extern "C" fn get_api_resources(out_len: *mut usize) -> *const Resource {
    static METHODS: [HttpMethod; 4] = [
        HttpMethod::Get, HttpMethod::Post,
        HttpMethod::Put, HttpMethod::Delete,
    ];
    let slice = static_resource("welcomemessage", &METHODS);
    unsafe { *out_len = slice.len(); }
    slice.as_ptr()
}
```

## Plugin Loading Process

The engine loads plugins through the following process:

1. The `engine::plugin_manager::PluginManager` resolves the plugin's binary path using platform-specific conventions
2. The dynamic library is loaded via `libloading::Library`
3. The `create_plugin` symbol is located and called to get the Plugin interface
4. API routes and functions are registered in the `engine_core::plugin_registry::PluginRegistry`
5. The plugin's `run` function is called with configuration passed via `PluginContext`

## Request Handling

The plugin handles API requests through its `handle_request` function, which receives an `ApiRequest` and returns an `ApiResponse`:

```rust
extern "C" fn handle_request(req: *const ApiRequest) -> *mut ApiResponse {
    if req.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let request = &*req;
        
        // Request validation
        if let Err(response) = validate_jwt_token(request) {
            return response;
        }

        // Path and method matching
        let path = CStr::from_ptr(request.path).to_str().unwrap_or("<invalid>");

        match request.method {
            HttpMethod::Get if path == "resource_name" => {
                json_response(200, r#"{"data": "example"}"#)
            }
            // Additional route handlers...
            _ => method_not_allowed_response(request.method, request.path),
        }
    }
}
```

## Communication Between Plugins

Plugins can communicate with each other using the WebSocket-based event system. Each plugin can:

1. Connect to the central WebSocket server via `WsClient::connect`
2. Subscribe to specific topics with `ws_client.subscribe()`
3. Publish events with `ws_client.publish()`

### Real-World Example: WiFi Plugin Communication Flow

The WiFi plugin demonstrates this communication pattern through a complete frontend-backend event cycle:

#### Step 1: Frontend Publishes Event to Backend
In the WiFi plugin frontend (`step-wifi.js`), after successfully connecting to a network:

```javascript
// Frontend publishes WiFi connection event
const published = appManager.publish('plugin_wifi', 'WifiCompleted', 
    { status: 'connected', ssid: 'MyNetwork' }
);
```

This uses the frontend's `appManager` to publish an event with:
- Source: `"plugin_wifi"` (the publishing plugin)
- Event name: `"WifiCompleted"` (the event type)
- Payload: Connection status and network details

#### Step 2: Backend Plugin Processes Event
The engine's WebSocket server receives this event and routes it to subscribers. In the backend:

```rust
// In the backend plugin's event handler
fn handle_wifi_event(event: &Event) -> Result<(), Error> {
    match event.name.as_str() {
        "WifiCompleted" => {
            // Parse event payload
            let payload = serde_json::from_value::<WifiStatus>(event.payload.clone())?;
            
            // Update system state based on WiFi status
            if payload.status == "connected" {
                // Store connection in system configuration
                config.set_wifi_network(payload.ssid);
                
                // Publish routing event for frontend
                ws_client.publish(&Event {
                    source: "engine",
                    name: "SwitchRoute",
                    payload: json!({ "route": "/status/web" }),
                })?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
```

#### Step 3: Backend Engine Publishes Routing Event
After processing the WiFi event, the engine publishes a routing event:

```rust
// Engine publishes navigation event
ws_client.publish(&Event {
    source: "engine",
    name: "SwitchRoute",
    payload: json!({ "route": "/status/web" }),
})?;
```

#### Step 4: Frontend Router Reacts to Routing Event
The frontend router component (`router.js`) is subscribed to these events:

```javascript
// In router.js - subscribing to routing events
appManager.subscribe("SwitchRoute", (event) => {
    if (event.payload && event.payload.route) {
        history.pushState({}, "", event.payload.route);
        window.dispatchEvent(new PopStateEvent("popstate"));
    }
});
```

This subscription allows the router to respond to routing events by changing the frontend route and triggering the appropriate view to be displayed.

### Technical Implementation

This event system uses WebSockets to maintain a persistent connection between frontend and backend components:

1. Backend plugins use `WsClient` to connect to the central event bus
2. Frontend components connect via the browser's WebSocket API
3. Events are serialized as JSON for cross-language compatibility
4. Event handlers in both frontend and backend process these messages

The combination of REST APIs (for direct commands) and WebSocket events (for notifications and state changes) creates a flexible architecture that supports both request-response and publish-subscribe communication patterns.

## Workflow Support

For plugins participating in longer workflows, the optional workflow functions allow:

- `run_workflow`: Initiate a background task
- `on_progress`: Report the task's progress
- `on_complete`: Signal task completion

The engine or frontend can use these to track workflow progress and orchestrate multi-step processes across different plugins.

## Plugin Metadata and Execution Plan

Plugins can be configured using metadata in an execution plan:

```rust
#[derive(Debug, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub plugin_route: String,
    pub version: String,
    pub plugin_location_type: String,
    pub plugin_base_path: String,
    // Additional metadata...
    pub run_after_event_name: Option<String>,
    pub completed_event_name: Option<String>,
}
```

This metadata defines not just the plugin's identity but also its position in execution workflows through events that trigger its execution and those it emits upon completion.