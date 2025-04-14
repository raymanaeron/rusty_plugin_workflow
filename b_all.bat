@echo off
echo Building plugin_wifi...
cargo build --manifest-path plugins\plugin_wifi\Cargo.toml

if errorlevel 1 (
    echo Failed to build plugin_wifi.
    exit /b 1
)

echo Building engine...
cargo build --manifest-path engine\Cargo.toml

if errorlevel 1 (
    echo Failed to build engine.
    exit /b 1
)

echo All builds successful.
