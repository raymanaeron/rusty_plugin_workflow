@echo off
REM This batch file deletes the top-level target folder

echo Deleting root target folder...
rmdir /s /q target

if exist target (
    echo Failed to delete target folder.
) else (
    echo Target folder successfully deleted.
)
