# Plugin: {{plugin_name}}

Route: `{{plugin_route}}`
Resource: `{{resource_name}}`

## HTTP API Endpoints

### GET /api/{{plugin_route}}/{{resource_name}}
Retrieves current configuration

**Response**
```json
{
    "field1": "string",
    "field2": boolean
}
```

### POST /api/{{plugin_route}}/{{resource_name}}
Creates new configuration

**Request Body**
```json
{
    "field1": "string",
    "field2": boolean
}
```

### PUT /api/{{plugin_route}}/{{resource_name}}
Updates existing configuration

**Request Body**
```json
{
    "field1": "string",
    "field2": boolean
}
```

### DELETE /api/{{plugin_route}}/{{resource_name}}
Resets configuration to defaults

## WebSocket Integration

### Topics
- `{{resource_name}}Updated` - Published when configuration changes

### Message Format
```json
{
    "publisher_name": "{{plugin_route}}_ui",
    "topic": "{{resource_name}}Updated",
    "payload": "JSON string of updated data",
    "timestamp": "ISO-8601 timestamp"
}
```

## File Structure
```
plugins/
└── {{plugin_name}}/
    ├── src/
    │   └── lib.rs
    ├── web/
    │   ├── step-{{plugin_route}}.html
    │   └── step-{{plugin_route}}.js
    └── Cargo.toml
```

## Integration
Add to engine/lib.rs or via execution_plan.toml:

```toml
[[plugins]]
name = "{{plugin_name}}"
plugin_route = "{{plugin_route}}"
version = "1.0.0"
plugin_location_type = "local"
plugin_base_path = "./plugins"
```