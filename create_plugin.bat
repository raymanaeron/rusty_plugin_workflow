@echo off
setlocal EnableDelayedExpansion

if "%3"=="" (
  echo Usage: create_plugin.bat ^<plugin_name^> ^<plugin_route^> ^<resource_name^>
  exit /b 1
)

set PLUGIN_NAME=%1
set PLUGIN_ROUTE=%2
set RESOURCE_NAME=%3

REM Convert resource_name to CamelCase
set "RESOURCE_NAME_CAMEL="
for %%w in (%RESOURCE_NAME:_= %) do (
    call :CapFirst %%w
    set "RESOURCE_NAME_CAMEL=!RESOURCE_NAME_CAMEL!!word!"
)

set TEMPLATE_DIR=plugin_templates
set TARGET_DIR=plugins\%PLUGIN_NAME%

echo Creating plugin: %PLUGIN_NAME% (route=%PLUGIN_ROUTE%, resource=%RESOURCE_NAME%, resource_camel=%RESOURCE_NAME_CAMEL%)

if not exist "%TARGET_DIR%\src" mkdir "%TARGET_DIR%\src"
if not exist "%TARGET_DIR%\web" mkdir "%TARGET_DIR%\web"

REM Process Cargo.toml
powershell -Command "(Get-Content '%TEMPLATE_DIR%\Cargo.toml.template') -replace '{{plugin_name}}','%PLUGIN_NAME%' | Set-Content '%TARGET_DIR%\Cargo.toml'"

REM Process lib.rs - use both resource_name and camelcased version
powershell -Command "(Get-Content '%TEMPLATE_DIR%\lib.rs.template') -replace '{{plugin_name}}','%PLUGIN_NAME%' -replace '{{plugin_route}}','%PLUGIN_ROUTE%' -replace '{{resource_name}}','%RESOURCE_NAME%' -replace '{{resource_name_camel}}','%RESOURCE_NAME_CAMEL%' | Set-Content '%TARGET_DIR%\src\lib.rs'"

REM Process HTML
powershell -Command "(Get-Content '%TEMPLATE_DIR%\step-template.html') -replace '{{plugin_route}}','%PLUGIN_ROUTE%' | Set-Content '%TARGET_DIR%\web\step-%PLUGIN_ROUTE%.html'"

REM Process JS
powershell -Command "(Get-Content '%TEMPLATE_DIR%\step-template.js') -replace '{{plugin_route}}','%PLUGIN_ROUTE%' -replace '{{resource_name}}','%RESOURCE_NAME%' -replace '{{resource_name_camel}}','%RESOURCE_NAME_CAMEL%' -replace '{{plugin_name}}','%PLUGIN_NAME%' | Set-Content '%TARGET_DIR%\web\step-%PLUGIN_ROUTE%.js'"

REM Create README.md
@echo # Plugin: %PLUGIN_NAME% > "%TARGET_DIR%\README.md"
@echo. >> "%TARGET_DIR%\README.md"
@echo Route: `%PLUGIN_ROUTE%` >> "%TARGET_DIR%\README.md"
@echo Resource: `%RESOURCE_NAME%` >> "%TARGET_DIR%\README.md"
@echo. >> "%TARGET_DIR%\README.md"
@echo ## File Structure >> "%TARGET_DIR%\README.md"
@echo. >> "%TARGET_DIR%\README.md"
@echo ```  >> "%TARGET_DIR%\README.md"
@echo plugins/ >> "%TARGET_DIR%\README.md"
@echo └── %PLUGIN_NAME%/ >> "%TARGET_DIR%\README.md"
@echo     ├── src/ >> "%TARGET_DIR%\README.md"
@echo     │   └── lib.rs >> "%TARGET_DIR%\README.md"
@echo     ├── web/ >> "%TARGET_DIR%\README.md"
@echo     │   ├── step-%PLUGIN_ROUTE%.html >> "%TARGET_DIR%\README.md"
@echo     │   └── step-%PLUGIN_ROUTE%.js >> "%TARGET_DIR%\README.md"
@echo     └── Cargo.toml >> "%TARGET_DIR%\README.md"
@echo ```  >> "%TARGET_DIR%\README.md"
@echo. >> "%TARGET_DIR%\README.md"
@echo ## Core Plugin Usage >> "%TARGET_DIR%\README.md"
@echo. >> "%TARGET_DIR%\README.md"
@echo To load this plugin in your engine/lib.rs: >> "%TARGET_DIR%\README.md"
@echo - Add it as a core plugin like other plugins >> "%TARGET_DIR%\README.md"
@echo - Or use execution plan loader with metadata from the plan >> "%TARGET_DIR%\README.md"

echo ✅ Plugin %PLUGIN_NAME% scaffolded under %TARGET_DIR%

goto :eof

:CapFirst
set "word=%1"
set "first=%word:~0,1%"
set "rest=%word:~1%"
for %%A in (A B C D E F G H I J K L M N O P Q R S T U V W X Y Z) do (
    if /i "%%A"=="%first%" set "first=%%A"
)
set "word=%first%%rest%"
goto :eof
