# {{plugin_name}}

This plugin implements the `{{plugin_route}}` step in the OOBE workflow. It exposes an HTTP API endpoint:

```
GET  /api/{{plugin_route}}/{{resource_name}}
POST /api/{{plugin_route}}/{{resource_name}}
```

## Folder structure

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

## Integration Notes

### Core Plugin Integration
Update `engine/src/lib.rs` to load and register this plugin manually using:

```rust
let (plugin, _lib) = load_plugin(plugin_utils::resolve_plugin_filename("{{plugin_name}}"))?;
(plugin.run)(&PluginContext { config: CString::new("").unwrap().as_ptr() });
registry.register(plugin);
```

### Dynamic Plan-Based Integration
Add the plugin to `execution_plan.toml`:

```toml
[[plugins]]
name = "{{plugin_name}}"
route = "{{plugin_route}}"
location_type = "Local"
location = "plugins/{{plugin_name}}/target/debug/{{plugin_name}}.dll"
```