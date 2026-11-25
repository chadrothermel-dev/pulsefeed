@echo off
setlocal
set "ROOT=%~dp0"
if "%PULSEFEED_PORT%"=="" set "PULSEFEED_PORT=8080"
set "PULSEFEED_DB=%ROOT%pulsefeed.db"

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
