# Plugin Creation Helper

This guide explains how to use the provided scripts to generate a new plugin using boilerplate templates.

## Files Provided

- `create_plugin.bat` – Windows batch script
- `create_plugin.sh` – Linux/macOS shell script

These scripts use a folder named `plugin_templates/` which contains the following files:

```
plugin_templates/
├── Cargo_template.toml
├── lib_template.rs
├── step_template.html
├── step_template.js
├── README_template.md
```

## Plugin Output Structure

The created plugin will look like this:

```
plugins/
└── plugin_example/
    ├── src/
    │   └── lib.rs
    ├── web/
    │   ├── step-example.html
    │   └── step-example.js
    ├── Cargo.toml
    └── README.md
```

## Usage

### On Windows

Run the script with your desired plugin name:
```cmd
create_plugin.bat plugin_wifi wifi network
```

This will:

- Create a folder `plugins/plugin_wifi`
- Use `plugin_wifi` as the plugin crate name
- Use `wifi` as the route name (`plugin_wifi` → `wifi`)
- Use `network` as the resource name

### On macOS/Linux

Make the script executable (first time only):
```bash
chmod +x create_plugin.sh
```

Run the script:
```bash
./create_plugin.sh plugin_wifi wifi network
```

Same as Windows – it will generate the required structure and templates using the `plugin_wifi` name.

## Parameter Explanation

| Parameter       | Meaning                                                                             |
|----------------|--------------------------------------------------------------------------------------|
| `plugin_wifi`  | Full plugin name. Also becomes the folder and crate name                             |
| `wifi`         | The route name for the plugin. Example: /wifi/web - refers to html and js location   |
| `network`      | The resource name. Example: /api/wifi/network - refers to a RESTful resource         |

These names will be inserted into:

- Rust module name
- Cargo.toml `[package]`
- Web assets like `step-wifi.html` and `step-wifi.js`

---

Make sure the `plugin_templates/` folder exists and includes the necessary template files.