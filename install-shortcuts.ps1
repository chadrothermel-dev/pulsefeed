$ErrorActionPreference = "SilentlyContinue"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$icoPath = Join-Path $root "pulsefeed.ico"
$launcherPs1 = Join-Path $root "launch-app.ps1"
$launcherVbs = Join-Path $root "launch-app.vbs"

# Always generate launch-app.vbs with the current install folder path so it works wherever extracted
@"
Set shell = CreateObject("WScript.Shell")
shell.Run "powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""$launcherPs1""", 0, False
"@ | Set-Content -Path $launcherVbs -Encoding ASCII

$wsh = New-Object -ComObject WScript.Shell
$desktopDir = [Environment]::GetFolderPath("Desktop")
$startMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"

foreach ($dir in @($desktopDir, $startMenuDir)) {
    if (-not $dir -or -not (Test-Path $dir)) { continue }
    $lnkPath = Join-Path $dir "PulseFeed.lnk"
    $sc = $wsh.CreateShortcut($lnkPath)
    $sc.TargetPath = "C:\Windows\System32\wscript.exe"
    $sc.Arguments = "`"$launcherVbs`""
    $sc.WorkingDirectory = $root
    if (Test-Path $icoPath) {
        $sc.IconLocation = "$icoPath,0"
    }
    $sc.Description = "PulseFeed Enterprise RSS & Intelligence Reader"
    $sc.Save()
}
