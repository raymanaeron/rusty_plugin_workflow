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

REM === Build desktop_ui  ===
echo Building desktop UI ...
cargo build --manifest-path engine_desktop_ui\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build desktop UI.
    exit /b 1
)

REM === Copy static web assets ===
echo Copying root web folder to engine output directory...
xcopy /E /I /Y webapp %TARGET%\webapp

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_terms\web %TARGET%\terms\web
if errorlevel 1 (
    echo Failed to copy plugin_terms web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_wifi\web %TARGET%\wifi\web
if errorlevel 1 (
    echo Failed to copy plugin_wifi web folder.
    exit /b 1
)

echo Copying the logger_config.json file to the engine output directory...
copy app_config.toml %TARGET%\app_config.toml
if errorlevel 1 (
    echo Failed to copy app_config.toml.
    exit /b 1
)

echo All builds successful.
endlocal
