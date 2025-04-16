@echo off
setlocal ENABLEEXTENSIONS

REM === Parse arguments ===
set MODE=debug
set CARGO_FLAG=
if "%1"=="--release" (
    set MODE=release
    set CARGO_FLAG=--release
)
set TARGET=target\%MODE%

echo [BUILD] Mode set to %MODE%
echo [BUILD] Output path: %TARGET%

REM === Build plugin_terms ===
echo Building plugin_terms...
cargo build --manifest-path plugins\plugin_terms\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_terms.
    exit /b 1
)

REM === Build plugin_wifi ===
echo Building plugin_wifi...
cargo build --manifest-path plugins\plugin_wifi\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_wifi.
    exit /b 1
)

REM === Build engine ===
echo Building engine...
cargo build --manifest-path engine\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build engine.
    exit /b 1
)

REM === Copy static web assets ===
echo Copying root web folder to engine output directory...
xcopy /E /I /Y webapp %TARGET%\webapp

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_terms\web %TARGET%\terms\web
xcopy /E /I /Y plugins\plugin_wifi\web %TARGET%\wifi\web

if errorlevel 1 (
    echo Failed to copy web folders.
    exit /b 1
)

echo All builds successful.
endlocal
