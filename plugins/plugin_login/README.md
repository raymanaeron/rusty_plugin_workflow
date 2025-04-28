# Plugin: plugin_login

Route: `login`  
Resource: `user`

## File Structure

```
plugins/
└── plugin_login/
    ├── src/
    │   └── lib.rs
    ├── web/
    │   ├── step-login.html
    │   └── step-login.js
    └── Cargo.toml
```

## Core Plugin Usage

To load this plugin in your engine/lib.rs:
- Add it as a core plugin like other plugins
- Or use execution plan loader with metadata from the plan

