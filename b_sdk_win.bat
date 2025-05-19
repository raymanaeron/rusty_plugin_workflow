@echo off
setlocal ENABLEEXTENSIONS

echo [SDK BUILD] Starting build process in release mode...
call .\b_win.bat --release
if errorlevel 1 (
    echo [SDK BUILD] Build failed!
    exit /b 1
)

echo [SDK BUILD] Build completed successfully.
echo [SDK BUILD] Creating SDK package...

REM Create SDK directory if it doesn't exist
if not exist "sdk" mkdir "sdk"

echo [SDK BUILD] Copying executable and configuration files...
copy /Y "target\release\engine_desktop_ui.exe" "sdk\"
copy /Y "app_config.toml" "sdk\"
copy /Y "execution_plan.toml" "sdk\"

plugin_provisioning

echo [SDK BUILD] Copying plugin DLLs...
copy /Y "target\release\plugin_welcome.dll" "sdk\"
copy /Y "target\release\plugin_wifi.dll" "sdk\"
copy /Y "target\release\plugin_execplan.dll" "sdk\"
copy /Y "target\release\plugin_login.dll" "sdk\"
copy /Y "target\release\plugin_provisioning.dll" "sdk\"
copy /Y "target\release\plugin_terms.dll" "sdk\"
copy /Y "target\release\plugin_finish.dll" "sdk\"

echo [SDK BUILD] Copying plugin web folders...
xcopy /E /I /Y "target\release\welcome" "sdk\welcome"
xcopy /E /I /Y "target\release\wifi" "sdk\wifi"
xcopy /E /I /Y "target\release\execution" "sdk\execution"
xcopy /E /I /Y "target\release\login" "sdk\login"
xcopy /E /I /Y "target\release\provision" "sdk\provision"
xcopy /E /I /Y "target\release\terms" "sdk\terms"
xcopy /E /I /Y "target\release\finish" "sdk\finish"

echo [SDK BUILD] Copying webapp folder...
xcopy /E /I /Y "target\release\webapp" "sdk\webapp"

echo [SDK BUILD] SDK package created successfully at .\sdk
echo [SDK BUILD] Done.

endlocal
