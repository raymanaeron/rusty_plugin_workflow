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

echo "[BUILD] Mode set to $MODE"
echo "[BUILD] Output path: $TARGET"

# === Build plugin_terms ===
echo "Building plugin_terms..."
cargo build --manifest-path plugins/plugin_terms/Cargo.toml $CARGO_FLAG

# === Build plugin_wifi ===
echo "Building plugin_wifi..."
cargo build --manifest-path plugins/plugin_wifi/Cargo.toml $CARGO_FLAG

# === Build engine ===
echo "Building engine..."
cargo build --manifest-path engine/Cargo.toml $CARGO_FLAG

# === Build engine_desktop_ui ===
echo "Building desktop UI..."
cargo build --manifest-path engine_desktop_ui/Cargo.toml $CARGO_FLAG

# === Copy static web assets ===
echo "Copying root web folder to engine output directory..."
rsync -a webapp/ "$TARGET/webapp/"

echo "Copying plugins web folder to engine output directory..."
rsync -a plugins/plugin_terms/web/ "$TARGET/terms/web/"
rsync -a plugins/plugin_wifi/web/ "$TARGET/wifi/web/"

echo "Copying the app config file to the engine output directory..."
cp app_config.toml "$TARGET/app_config.toml"

echo "✅ All builds successful."
