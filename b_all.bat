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

echo Copying root web folder to engine output directory...
xcopy /E /I /Y webapp target\debug\webapp

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_wifi\web target\debug\wifi\web

if errorlevel 1 (
    echo Failed to copy web folder.
    exit /b 1
)

echo All builds successful.
