# Plugin Creation Helper

This guide explains how to use the provided scripts to generate a new plugin using boilerplate templates.

## Files Provided

- `create_plugin.bat` – Windows batch script
- `create_plugin.sh` – Linux/macOS shell script

These scripts use a folder named `plugin_templates/` which contains the following files:

```
plugin_templates/
├── Cargo.toml.template
├── lib.rs.template
├── step.html.template
├── step.js.template
└── README.md.template
```

## Plugin Output Structure

The created plugin will look like this:

```
plugins/
└── plugin_example/
    ├── src/
    │   └── lib.rs
    ├── web/
    │   ├── index.html
    │   └── main.js
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
| `wifi`         | The plugin route name. Example: /wifi/web - refers to HTML and JS location           |
| `network`      | The resource name. Used in API endpoints as /api/wifi/network                        |

These names will be used in the templates as:

- `{{plugin_name}}` - The full plugin name (e.g., plugin_wifi)
- `{{plugin_route}}` - The route name (e.g., wifi)
- `{{resource_name}}` - The resource name (e.g., network)

These will be inserted into:
- Cargo.toml package name
- lib.rs constants and resource handlers
- Web assets

---

Make sure the `plugin_templates/` folder exists and includes all necessary template files before running the scripts.