$ErrorActionPreference = "SilentlyContinue"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$port = 8080
$url = "http://127.0.0.1:$port"

# Check if PulseFeed is already listening on port 8080
$running = Test-NetConnection -ComputerName 127.0.0.1 -Port $port -InformationLevel Quiet -WarningAction SilentlyContinue
if (-not $running) {
    $exe = if (Test-Path (Join-Path $root "pulsefeed.exe")) {
        Join-Path $root "pulsefeed.exe"
    } else {
        Join-Path $root "backend\target\release\pulsefeed.exe"
    }
    $staticDir = if (Test-Path (Join-Path $root "dist")) {
        Join-Path $root "dist"
    } else {
        Join-Path $root "frontend\dist"
    }

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.WorkingDirectory = $root
    $psi.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden
    $psi.CreateNoWindow = $true
    $psi.UseShellExecute = $false
    $psi.EnvironmentVariables["PULSEFEED_STATIC_DIR"] = $staticDir
    $psi.EnvironmentVariables["PULSEFEED_DB"] = (Join-Path $root "pulsefeed.db")
    $psi.EnvironmentVariables["PULSEFEED_PORT"] = "$port"
    $psi.EnvironmentVariables["PULSEFEED_EDITION"] = if ($env:PULSEFEED_EDITION) { $env:PULSEFEED_EDITION } else { "personal" }
    [System.Diagnostics.Process]::Start($psi) | Out-Null

    # Wait briefly for server to accept connections
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Milliseconds 150
        if (Test-NetConnection -ComputerName 127.0.0.1 -Port $port -InformationLevel Quiet -WarningAction SilentlyContinue) { break }
    }
}

$chrome = @(
    "C:\Program Files\Google\Chrome\Application\chrome.exe",
    "C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    "$env:LOCALAPPDATA\Google\Chrome\Application\chrome.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($chrome) {
    Start-Process -FilePath $chrome -ArgumentList "--app=$url"
} else {
    Start-Process $url
}
