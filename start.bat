@echo off
setlocal
set "ROOT=%~dp0"
if "%PULSEFEED_PORT%"=="" set "PULSEFEED_PORT=8080"
if "%PULSEFEED_EDITION%"=="" set "PULSEFEED_EDITION=personal"
set "PULSEFEED_DB=%ROOT%pulsefeed.db"

REM Automatically create/update Desktop and Start Menu shortcuts with the PulseFeed icon
if exist "%ROOT%install-shortcuts.ps1" (
    powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "%ROOT%install-shortcuts.ps1"
)

if exist "%ROOT%pulsefeed.exe" (
    set "PULSEFEED_STATIC_DIR=%ROOT%dist"
    set "EXE=%ROOT%pulsefeed.exe"
) else (
    set "PULSEFEED_STATIC_DIR=%ROOT%frontend\dist"
    set "EXE=%ROOT%backend\target\release\pulsefeed.exe"
)

echo Starting PulseFeed Enterprise RSS Reader at http://127.0.0.1:%PULSEFEED_PORT%
start "" "http://127.0.0.1:%PULSEFEED_PORT%"
"%EXE%"
