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

REM === Build plugin_welcome ===
echo Building libjwt...
cargo build --manifest-path libjwt\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build libjwt.
    exit /b 1
)

REM === Build plugin_welcome ===
echo Building plugin_welcome...
cargo build --manifest-path plugins\plugin_welcome\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_welcome.
    exit /b 1
)

REM === Build plugin_mockwifi ===
echo Building plugin_mockwifi...
cargo build --manifest-path plugins\plugin_mockwifi\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_mockwifi.
    exit /b 1
)

REM === Build plugin_execplan ===
echo Building plugin_execplan...
cargo build --manifest-path plugins\plugin_execplan\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_execplan.
    exit /b 1
)

REM === Build plugin_login ===
echo Building plugin_login...
cargo build --manifest-path plugins\plugin_login\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_login.
    exit /b 1
)

REM === Build plugin_provisioning ===
echo Building plugin_provisioning...
cargo build --manifest-path plugins\plugin_provisioning\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_provisioning.
    exit /b 1
)

REM === Build plugin_terms ===
echo Building plugin_terms...
cargo build --manifest-path plugins\plugin_terms\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_terms.
    exit /b 1
)

REM === Build plugin_settings ===
echo Building plugin_settings...
cargo build --manifest-path plugins\plugin_settings\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_settings.
    exit /b 1
)

REM === Build plugin_status ===
echo Building plugin_status...
cargo build --manifest-path plugins\plugin_status\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_status.
    exit /b 1
)

REM === Build plugin_howto ===
echo Building plugin_howto...
cargo build --manifest-path plugins\plugin_howto\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_howto.
    exit /b 1
)
REM === Build plugin_tutorial ===
echo Building plugin_tutorial...
cargo build --manifest-path plugins\plugin_tutorial\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_tutorial.
    exit /b 1
)

REM === Build plugin_finish ===
echo Building plugin_finish...
cargo build --manifest-path plugins\plugin_finish\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_finish.
    exit /b 1
)

REM === Build plugin_task_agent_headless ===
echo Building plugin_task_agent_headless...
cargo build --manifest-path plugins\plugin_task_agent_headless\Cargo.toml %CARGO_FLAG%
if errorlevel 1 (
    echo Failed to build plugin_task_agent_headless.
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
xcopy /E /I /Y plugins\plugin_welcome\web %TARGET%\welcome\web
if errorlevel 1 (
    echo Failed to copy plugin_welcome web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_mockwifi\web %TARGET%\mwifi\web
if errorlevel 1 (
    echo Failed to copy plugin_mockwifi web folder.
    exit /b 1
)

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_execplan\web %TARGET%\execution\web
if errorlevel 1 (
    echo Failed to copy plugin_execplan web folder.
    exit /b 1
)

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_login\web %TARGET%\login\web
if errorlevel 1 (
    echo Failed to copy plugin_login web folder.
    exit /b 1
)

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_provisioning\web %TARGET%\provision\web
if errorlevel 1 (
    echo Failed to copy plugin_provisioning web folder.
    exit /b 1
)

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_terms\web %TARGET%\terms\web
if errorlevel 1 (
    echo Failed to copy plugin_terms web folder.
    exit /b 1
)

echo Copying plugins web folder to engine output directory...
xcopy /E /I /Y plugins\plugin_settings\web %TARGET%\settings\web
if errorlevel 1 (
    echo Failed to copy plugin_settings web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_status\web %TARGET%\status\web
if errorlevel 1 (
    echo Failed to copy plugin_status web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_howto\web %TARGET%\howto\web
if errorlevel 1 (
    echo Failed to copy plugin_howto web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_tutorial\web %TARGET%\tutorial\web
if errorlevel 1 (
    echo Failed to copy plugin_tutorial web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_finish\web %TARGET%\finish\web
if errorlevel 1 (
    echo Failed to copy plugin_finish web folder.
    exit /b 1
)

xcopy /E /I /Y plugins\plugin_task_agent_headless\web %TARGET%\taskagent\web
if errorlevel 1 (
    echo Failed to copy plugin_task_agent_headless web folder.
    exit /b 1
)

echo Copying the app config file to the engine output directory...
copy app_config.toml %TARGET%\app_config.toml
if errorlevel 1 (
    echo Failed to copy app_config.toml.
    exit /b 1
)

echo Copying the execution_plan.toml file to the engine output directory...
copy execution_plan.toml %TARGET%\execution_plan.toml 
if errorlevel 1 (
    echo Failed to copy  execution_plan.toml.
    exit /b 1
)

echo Copying the external execution_plan.toml file to the engine output directory...

if not exist "%TARGET%\ext_plan" mkdir "%TARGET%\ext_plan"
if not exist "%TARGET%\ext_plan\Echo" mkdir "%TARGET%\ext_plan\Echo"
if not exist "%TARGET%\ext_plan\Echo\1.3" mkdir "%TARGET%\ext_plan\Echo\1.3"

copy /Y ".\ext_plan\Echo\1.3\execution_plan.toml" "%TARGET%\ext_plan\Echo\1.3\execution_plan.toml"
if errorlevel 1 (
    echo Failed to copy  external execution_plan.toml.
    exit /b 1
)

echo All builds successful.
endlocal
