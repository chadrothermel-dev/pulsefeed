$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

# Ensure Desktop & Start Menu shortcuts with the PulseFeed icon are installed
$shortcutScript = Join-Path $root "install-shortcuts.ps1"
if (Test-Path $shortcutScript) {
    & $shortcutScript
}

# Support both standalone ZIP layout (./pulsefeed.exe + ./dist) and source repo layout
$standaloneExe = Join-Path $root "pulsefeed.exe"
$sourceExe = Join-Path $root "backend\target\release\pulsefeed.exe"

if (Test-Path $standaloneExe) {
    $exe = $standaloneExe
    $staticDir = Join-Path $root "dist"
} elseif (Test-Path $sourceExe) {
    $exe = $sourceExe
    $staticDir = Join-Path $root "frontend\dist"
} else {
    Write-Host "Building PulseFeed release binary from source..." -ForegroundColor Cyan
    Push-Location (Join-Path $root "backend")
    cargo build --release
    Pop-Location
    $exe = $sourceExe
    $staticDir = Join-Path $root "frontend\dist"
}

$env:PULSEFEED_STATIC_DIR = $staticDir
$env:PULSEFEED_DB = Join-Path $root "pulsefeed.db"
if (-not $env:PULSEFEED_PORT) { $env:PULSEFEED_PORT = "8080" }
if (-not $env:PULSEFEED_EDITION) { $env:PULSEFEED_EDITION = "personal" }

Write-Host "Starting PulseFeed ($($env:PULSEFEED_EDITION) edition) at http://127.0.0.1:$($env:PULSEFEED_PORT)" -ForegroundColor Green
& $exe
