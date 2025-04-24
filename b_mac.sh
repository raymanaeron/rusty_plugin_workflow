#!/bin/bash
set -e  # Exit immediately on any error

# === Parse arguments ===
MODE="debug"
CARGO_FLAG=""
if [[ "$1" == "--release" ]]; then
  MODE="release"
  CARGO_FLAG="--release"
fi
TARGET="target/$MODE"

echo ">> TARGET is: $TARGET"

echo "[BUILD] Mode set to $MODE"
echo "[BUILD] Output path: $TARGET"

# === Build plugin_terms ===
echo "Building plugin_terms..."
cargo build --manifest-path plugins/plugin_terms/Cargo.toml $CARGO_FLAG

# === Build plugin_wifi ===
echo "Building plugin_wifi..."
cargo build --manifest-path plugins/plugin_wifi/Cargo.toml $CARGO_FLAG

# === Build plugin_status ===
echo "Building plugin_status..."
cargo build --manifest-path plugins/plugin_status/Cargo.toml $CARGO_FLAG

# === Build plugin_task_agent_headless ===
echo "Building plugin_task_agent_headless..."
cargo build --manifest-path plugins/plugin_task_agent_headless/Cargo.toml $CARGO_FLAG

# === Build plugin_settings ===
echo "Building plugin_settings..."
cargo build --manifest-path plugins/plugin_settings/Cargo.toml $CARGO_FLAG

# === Build log_viewer ===
echo "Building log_viewer..."
cargo build --manifest-path log_viewer/Cargo.toml $CARGO_FLAG

# === Build engine ===
echo "Building engine..."
cargo build --manifest-path engine/Cargo.toml $CARGO_FLAG

# === Build engine_desktop_ui ===
echo "Building desktop UI..."
cargo build --manifest-path engine_desktop_ui/Cargo.toml $CARGO_FLAG

echo "📦 TARGET is: $TARGET"
ls -ld "$TARGET"

# === Create destination folders before rsync ===
echo "Creating plugin destination folders..."
mkdir -p "$TARGET/terms/web"
mkdir -p "$TARGET/wifi/web"
mkdir -p "$TARGET/status/web"
mkdir -p "$TARGET/taskagent/web"
mkdir -p "$TARGET/settings/web"

# === Copy static web assets ===
echo "Copying root web folder to engine output directory..."
rsync -a webapp/ "$TARGET/webapp/"

echo "Copying plugins web folder to engine output directory..."
rsync -a plugins/plugin_terms/web/ "$TARGET/terms/web/"
rsync -a plugins/plugin_wifi/web/ "$TARGET/wifi/web/"
rsync -a plugins/plugin_status/web/ "$TARGET/status/web/"
rsync -a plugins/plugin_task_agent_headless/web/ "$TARGET/taskagent/web/"
rsync -a plugins/plugin_settings/web/ "$TARGET/settings/web/"

# === Copy plugin dylibs to output folder (macOS only) ===
echo "Copying plugin shared libraries to engine output directory..."
cp "$TARGET/libplugin_terms.dylib" "$TARGET/plugin_terms.dylib"
cp "$TARGET/libplugin_wifi.dylib" "$TARGET/plugin_wifi.dylib"
cp "$TARGET/libplugin_status.dylib" "$TARGET/plugin_status.dylib"
cp "$TARGET/libplugin_task_agent_headless.dylib" "$TARGET/plugin_task_agent_headless.dylib"
cp "$TARGET/libplugin_settings.dylib" "$TARGET/plugin_settings.dylib"

echo "Copying the app config file to the engine output directory..."
cp app_config.toml "$TARGET/app_config.toml"

echo "Copying the execution_plan.toml file to the engine output directory..."
cp execution_plan.toml "$TARGET/execution_plan.toml"

echo "All builds successful."
