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

# === Build all plugins and engine components ===
echo "Building plugin_welcome..."
cargo build --manifest-path plugins/plugin_welcome/Cargo.toml $CARGO_FLAG

echo "Building plugin_mockwifi..."
cargo build --manifest-path plugins/plugin_mockwifi/Cargo.toml $CARGO_FLAG

echo "Building plugin_execution..."
cargo build --manifest-path plugins/plugin_execplan/Cargo.toml $CARGO_FLAG

echo "Building plugin_login..."
cargo build --manifest-path plugins/plugin_login/Cargo.toml $CARGO_FLAG

echo "Building plugin_provisioning..."
cargo build --manifest-path plugins/plugin_provisioning/Cargo.toml $CARGO_FLAG

echo "Building plugin_terms..."
cargo build --manifest-path plugins/plugin_terms/Cargo.toml $CARGO_FLAG

echo "Building plugin_settings..."
cargo build --manifest-path plugins/plugin_settings/Cargo.toml $CARGO_FLAG

echo "Building plugin_status..."
cargo build --manifest-path plugins/plugin_status/Cargo.toml $CARGO_FLAG

echo "Building plugin_howto..."
cargo build --manifest-path plugins/plugin_howto/Cargo.toml $CARGO_FLAG

echo "Building plugin_tutorial..."
cargo build --manifest-path plugins/plugin_tutorial/Cargo.toml $CARGO_FLAG

echo "Building plugin_finish..."
cargo build --manifest-path plugins/plugin_finish/Cargo.toml $CARGO_FLAG

echo "Building plugin_task_agent_headless..."
cargo build --manifest-path plugins/plugin_task_agent_headless/Cargo.toml $CARGO_FLAG

echo "Building engine..."
cargo build --manifest-path engine/Cargo.toml $CARGO_FLAG

echo "Building engine_desktop_ui..."
cargo build --manifest-path engine_desktop_ui/Cargo.toml $CARGO_FLAG

echo "TARGET is: $TARGET"
ls -ld "$TARGET"

# === Create destination folders before rsync/cp ===
echo "Creating plugin destination folders..."
mkdir -p "$TARGET/welcome/web"
mkdir -p "$TARGET/mwifi/web"
mkdir -p "$TARGET/execution/web"
mkdir -p "$TARGET/login/web"
mkdir -p "$TARGET/provision/web"
mkdir -p "$TARGET/terms/web"
mkdir -p "$TARGET/settings/web"
mkdir -p "$TARGET/status/web"
mkdir -p "$TARGET/howto/web"
mkdir -p "$TARGET/tutorial/web"
mkdir -p "$TARGET/finish/web"
mkdir -p "$TARGET/taskagent/web"
mkdir -p "$TARGET/webapp"

# === Copy static web assets ===
echo "Copying root web folder to engine output directory..."
rsync -a webapp/ "$TARGET/webapp/"

echo "Copying plugins web folder to engine output directory..."
rsync -a plugins/plugin_welcome/web/ "$TARGET/welcome/web/"
rsync -a plugins/plugin_mockwifi/web/ "$TARGET/mwifi/web/"
rsync -a plugins/plugin_execplan/web/ "$TARGET/execution/web/"
rsync -a plugins/plugin_login/web/ "$TARGET/login/web/"
rsync -a plugins/plugin_provisioning/web/ "$TARGET/provision/web/"
rsync -a plugins/plugin_terms/web/ "$TARGET/terms/web/"
rsync -a plugins/plugin_settings/web/ "$TARGET/settings/web/"
rsync -a plugins/plugin_status/web/ "$TARGET/status/web/"
rsync -a plugins/plugin_howto/web/ "$TARGET/howto/web/"
rsync -a plugins/plugin_tutorial/web/ "$TARGET/tutorial/web/"
rsync -a plugins/plugin_finish/web/ "$TARGET/finish/web/"
rsync -a plugins/plugin_task_agent_headless/web/ "$TARGET/taskagent/web/"


# === Copy plugin dylibs to output folder (macOS only) ===
echo "Copying plugin shared libraries to engine output directory..."
cp "$TARGET/libplugin_welcome.dylib" "$TARGET/plugin_welcome.dylib"
cp "$TARGET/libplugin_mockwifi.dylib" "$TARGET/plugin_mockwifi.dylib"
cp "$TARGET/libplugin_execplan.dylib" "$TARGET/plugin_execplan.dylib"
cp "$TARGET/libplugin_login.dylib" "$TARGET/plugin_login.dylib"
cp "$TARGET/libplugin_provisioning.dylib" "$TARGET/plugin_provisioning.dylib"
cp "$TARGET/libplugin_terms.dylib" "$TARGET/plugin_terms.dylib"
cp "$TARGET/libplugin_settings.dylib" "$TARGET/plugin_settings.dylib"
cp "$TARGET/libplugin_status.dylib" "$TARGET/plugin_status.dylib"
cp "$TARGET/libplugin_howto.dylib" "$TARGET/plugin_howto.dylib"
cp "$TARGET/libplugin_tutorial.dylib" "$TARGET/plugin_tutorial.dylib"
cp "$TARGET/libplugin_finish.dylib" "$TARGET/plugin_finish.dylib"
cp "$TARGET/libplugin_task_agent_headless.dylib" "$TARGET/plugin_task_agent_headless.dylib"

# === Copy config and plan files ===
echo "Copying the app config file to the engine output directory..."
cp app_config.toml "$TARGET/app_config.toml"

echo "Copying the execution_plan.toml file to the engine output directory..."
cp execution_plan.toml "$TARGET/execution_plan.toml"

echo "Copying the external execution_plan.toml file to the engine output directory..."
mkdir -p "$TARGET/ext_plan"
mkdir -p "$TARGET/ext_plan/Echo"
mkdir -p "$TARGET/ext_plan/Echo/1.3"

cp ./ext_plan/Echo/1.3/execution_plan.toml "$TARGET/ext_plan/Echo/1.3/execution_plan.toml"

echo "All builds successful."
