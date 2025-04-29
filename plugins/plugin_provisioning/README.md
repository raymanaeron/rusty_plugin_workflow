# Plugin: plugin_provisioning

Route: `provision`
Resource: `device`
Type: `Device`

## HTTP API Endpoints

### GET /api/provision/device
Retrieves all resources or a specific resource by ID

**Example Request (All Resources)**
```javascript
// JavaScript Client Example
async function getData() {
  try {
    const response = await fetch('/api/provision/device');
    if (response.ok) {
      const data = await response.json();
      console.log('Resources loaded:', data);
      return data;
    } else {
      console.error('Failed to load resources:', response.statusText);
      throw new Error(`Failed to load: ${response.statusText}`);
    }
  } catch (error) {
    console.error('Error loading resources:', error);
    throw error;
  }
}
```

**Example Request (Single Resource)**
```javascript
// JavaScript Client Example
async function getResource(id) {
  try {
    const response = await fetch(`/api/provision/device/${id}`);
    if (response.ok) {
      const data = await response.json();
      console.log('Resource loaded:', data);
      return data;
    } else {
      console.error('Failed to load resource:', response.statusText);
      throw new Error(`Failed to load: ${response.statusText}`);
    }
  } catch (error) {
    console.error('Error loading resource:', error);
    throw error;
  }
}
```

**Response**
```json
{
  "id": "string",
  "field1": "string",
  "field2": true
}
```

### POST /api/provision/device
Creates a new resource

**Example Request**
```javascript
// JavaScript Client Example
async function createResource(payload) {
  try {
    const response = await fetch('/api/provision/device', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    
    const data = await response.json();
    if (response.ok) {
      console.log('Resource created successfully:', data);
      return data;
    } else {
      console.error('Failed to create resource:', data);
      throw new Error(data.message || 'Failed to create resource');
    }
  } catch (error) {
    console.error('Error creating resource:', error);
    throw error;
  }
}
```

**Request Body**
```json
{
  "field1": "string",
  "field2": true
}
```

**Response**
```json
{
  "message": "Resource created",
  "id": "generated-id-string"
}
```

### PUT /api/provision/device/{id}
Updates an existing resource (complete replacement)

**Example Request**
```javascript
// JavaScript Client Example
async function updateResource(id, payload) {
  try {
    const response = await fetch(`/api/provision/device/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    
    if (response.ok) {
      const data = await response.json();
      console.log('Resource updated successfully:', data);
      return data;
    } else {
      const errorData = await response.json();
      console.error('Failed to update resource:', errorData);
      throw new Error(errorData.message || 'Failed to update resource');
    }
  } catch (error) {
    console.error('Error updating resource:', error);
    throw error;
  }
}
```

**Request Body**
```json
{
  "field1": "new value",
  "field2": false
}
```

### PATCH /api/provision/device/{id}
Updates an existing resource (partial update)

**Example Request**
```javascript
// JavaScript Client Example
async function patchResource(id, partialPayload) {
  try {
    const response = await fetch(`/api/provision/device/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(partialPayload)
    });
    
    if (response.ok) {
      const data = await response.json();
      console.log('Resource patched successfully:', data);
      return data;
    } else {
      const errorData = await response.json();
      console.error('Failed to patch resource:', errorData);
      throw new Error(errorData.message || 'Failed to patch resource');
    }
  } catch (error) {
    console.error('Error patching resource:', error);
    throw error;
  }
}
```

**Request Body**
```json
{
  "field1": "updated value"
}
```

### DELETE /api/provision/device/{id}
Deletes a specific resource

**Example Request**
```javascript
// JavaScript Client Example
async function deleteResource(id) {
  try {
    const response = await fetch(`/api/provision/device/${id}`, {
      method: 'DELETE'
    });
    
    if (response.ok) {
      const data = await response.json();
      console.log('Resource deleted successfully:', data);
      return data;
    } else {
      const errorData = await response.json();
      console.error('Failed to delete resource:', errorData);
      throw new Error(errorData.message || 'Failed to delete resource');
    }
  } catch (error) {
    console.error('Error deleting resource:', error);
    throw error;
  }
}
```

## WebSocket Integration

### Topics
- `DeviceUpdated` - Published when resource changes
- `DeviceCompleted` - Published when plugin operation completes

### Publishing Events

#### From JavaScript

```javascript
// In your step-provision.js file
export async function activate(container, appManager) {
  appManager.registerPlugin('plugin_provisioning');
  
  // Example publishing a completion event
  function publishCompletion() {
    const published = appManager.publish(
      'plugin_provisioning',                   // Publisher name
      'DeviceCompleted',  // Topic
      { status: 'completed', data: { /* additional data */ } }
    );
    
    if (published) {
      console.log("[plugin_provisioning] Completion status published");
    } else {
      console.warn("[plugin_provisioning] Completion publish failed");
    }
  }
  
  // Publishing from a button click handler
  continueBtn.addEventListener('click', () => {
    publishCompletion();
  });
}
```

#### From Rust

```rust
// In your lib.rs file
if let Some(client) = unsafe { &PLUGIN_WS_CLIENT } {
    RUNTIME.spawn(async move {
        if let Ok(mut ws_client) = client.lock() {
            let timestamp = chrono::Utc::now().to_rfc3339();
            let payload = r#"{"status": "completed", "data": {}}"#;
            
            // Publish an event
            if let Err(e) = ws_client.publish(
                "plugin_provisioning", 
                "DeviceCompleted", 
                payload,
                &timestamp
            ).await {
                eprintln!("[plugin_provisioning] Failed to publish: {}", e);
            } else {
                println!("[plugin_provisioning] Successfully published completion event");
            }
        }
    });
}
```

### Subscribing to Events

#### In JavaScript

```javascript
// In your step-provision.js file
export async function activate(container, appManager) {
  appManager.registerPlugin('plugin_provisioning');
  
  // Subscribe to a topic
  appManager.subscribe('plugin_provisioning', 'DeviceUpdated', (data) => {
    console.log('Received update:', data);
    
    // Update UI based on the data
    if (data.status === 'updated') {
      // Handle data update
      updateDisplay(data);
    } else if (data.status === 'deleted') {
      // Handle deletion
      removeFromDisplay(data.id);
    }
  });
  
  // Clean up when done
  return () => {
    appManager.unregisterPlugin('plugin_provisioning');
  };
}
```

#### In Rust

```rust
// In your lib.rs or engine code
pub async fn create_ws_plugin_client() {
    if let Ok(client) = WsClient::connect("plugin_provisioning", "ws://127.0.0.1:8081/ws").await {
        let client = Arc::new(Mutex::new(client));
        
        if let Ok(mut ws_client) = client.lock() {
            // Subscribe to a specific topic
            ws_client.subscribe("plugin_provisioning", "DeviceUpdated", "").await;
            println!("[plugin_provisioning] Subscribed to DeviceUpdated");
            
            // Register a callback for messages on that topic
            ws_client.on_message("DeviceUpdated", |msg| {
                println!("[plugin_provisioning] Received update: {}", msg);
                // Process the message here
            });
        }
        
        // Store the client for later use
        unsafe {
            PLUGIN_WS_CLIENT = Some(client);
        }
    }
}
```

## Workflow Integration

### Publishing Completion Events

When your plugin completes its task, it should publish a completion event that other plugins or the engine can subscribe to for workflow progression:

```javascript
// From a button click or automated process
function completeSetup() {
  // Do final processing...
  
  // Publish completion
  const published = appManager.publish('plugin_provisioning', 'DeviceCompleted', 
    { status: 'completed' }
  );
  
  if (published) {
    resultBox.innerHTML = '<div class="alert alert-success">Setup completed! Redirecting...</div>';
  } else {
    resultBox.innerHTML = '<div class="alert alert-warning">Failed to publish completion. Please try again.</div>';
  }
}
```

### Skipping a Step

To implement a skip function:

```javascript
// Skip button handler
if (skipBtn) {
  skipBtn.addEventListener('click', async () => {
    const published = appManager.publish('plugin_provisioning', 'DeviceCompleted', 
      { status: 'skipped' }
    );
    
    if (published) {
      console.log("[plugin_provisioning] Skip status published");
      resultBox.innerHTML = '<div class="alert alert-info">Setup skipped. Redirecting...</div>';
    } else {
      console.warn("[plugin_provisioning] Skip publish failed");
      resultBox.innerHTML = '<div class="alert alert-warning">Failed to publish skip status</div>';
    }
  });
}
```

## File Structure
```
plugins/
└── plugin_provisioning/
    ├── src/
    │   └── lib.rs              # Rust plugin implementation
    ├── web/
    │   ├── step-provision.html  # HTML UI template
    │   ├── step-provision.js    # JavaScript logic
    │   └── icons/               # Optional icons folder
    └── Cargo.toml              # Rust dependencies and metadata
```

## Integration
Add to engine/lib.rs or via execution_plan.toml:

```toml
[[plugins]]
name = "plugin_provisioning"
plugin_route = "provision"
version = "1.0.0"
plugin_location_type = "local"
plugin_base_path = "./plugins"
```
