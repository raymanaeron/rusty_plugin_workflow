@echo off
setlocal ENABLEEXTENSIONS

if "%~3"=="" (
  echo Usage: create_plugin.bat plugin_name plugin_route resource_name
  exit /b 1
)

set PLUGIN_NAME=%~1
set PLUGIN_ROUTE=%~2
set RESOURCE_NAME=%~3

set TEMPLATE_DIR=plugin_templates
set TARGET_DIR=plugins\%PLUGIN_NAME%

echo Creating plugin: %PLUGIN_NAME% (route=%PLUGIN_ROUTE%, resource=%RESOURCE_NAME%)
mkdir %TARGET_DIR%\src
mkdir %TARGET_DIR%\web

rem Process Cargo.toml
powershell -Command "(Get-Content %TEMPLATE_DIR%\Cargo.toml.template) -replace '{{plugin_name}}','%PLUGIN_NAME%' | Set-Content %TARGET_DIR%\Cargo.toml"

rem Process lib.rs
powershell -Command "(Get-Content %TEMPLATE_DIR%\lib.rs.template) -replace '{{plugin_name}}','%PLUGIN_NAME%' -replace '{{plugin_route}}','%PLUGIN_ROUTE%' -replace '{{resource_name}}','%RESOURCE_NAME%' | Set-Content %TARGET_DIR%\src\lib.rs"

rem Process HTML
powershell -Command "(Get-Content %TEMPLATE_DIR%\step-template.html) -replace '{{plugin_route}}','%PLUGIN_ROUTE%' | Set-Content %TARGET_DIR%\web\step-%PLUGIN_ROUTE%.html"

rem Process JS
powershell -Command "(Get-Content %TEMPLATE_DIR%\step-template.js) -replace '{{plugin_route}}','%PLUGIN_ROUTE%' -replace '{{resource_name}}','%RESOURCE_NAME%' | Set-Content %TARGET_DIR%\web\step-%PLUGIN_ROUTE%.js"

rem Create README
(
echo # Plugin: %PLUGIN_NAME%
echo.
echo Route: `%PLUGIN_ROUTE%`
echo Resource: `%RESOURCE_NAME%`
echo.
echo ## File Structure
echo ```
echo plugins\
echo └── %PLUGIN_NAME%\
echo     ├── src\
echo     │   └── lib.rs
echo     ├── web\
echo     │   ├── step-%PLUGIN_ROUTE%.html
echo     │   └── step-%PLUGIN_ROUTE%.js
echo     └── Cargo.toml
echo ```
echo.
echo ## Core Plugin Usage
echo Add to engine/lib.rs like existing plugins or via execution_plan.toml
) > %TARGET_DIR%\README.md

echo ✅ Plugin %PLUGIN_NAME% created at %TARGET_DIR%
