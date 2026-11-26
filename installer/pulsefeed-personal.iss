; PulseFeed Personal Edition — Windows Installer Script (Inno Setup 6)
#define MyAppName "PulseFeed"
#define MyAppVersion "1.2.0"
#define MyAppPublisher "PulseFeed"
#define MyAppURL "https://github.com/chadrothermel-dev/pulsefeed"
#define MyAppExeName "pulsefeed.exe"

[Setup]
AppId={{8D6E4B1A-4C92-4E21-9A77-3B8F2C109E44}
AppName={#MyAppName} Personal
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} Personal {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\PulseFeed
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist-installer
OutputBaseFilename=PulseFeed-Personal-Setup-v{#MyAppVersion}
SetupIconFile=..\pulsefeed.ico
UninstallDisplayIcon={app}\pulsefeed.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "startmenuicon"; Description: "Create a Start Menu shortcut"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\backend\target\release\pulsefeed.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\frontend\dist\*"; DestDir: "{app}\dist"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\pulsefeed.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\launch-app.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\install-shortcuts.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\start.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\start-enterprise.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\start.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "C:\Windows\System32\wscript.exe"; Parameters: """{app}\launch-app.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\pulsefeed.ico"; Comment: "PulseFeed Personal RSS & Intelligence Reader"; Tasks: startmenuicon
Name: "{autodesktop}\{#MyAppName}"; Filename: "C:\Windows\System32\wscript.exe"; Parameters: """{app}\launch-app.vbs"""; WorkingDir: "{app}"; IconFilename: "{app}\pulsefeed.ico"; Comment: "PulseFeed Personal RSS & Intelligence Reader"; Tasks: desktopicon

[Run]
; Generate local launch-app.vbs pointing to the chosen {app} directory
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command ""Set-Content -Path '{app}\launch-app.vbs' -Encoding ASCII -Value ('Set shell = CreateObject(""WScript.Shell"")' + [Environment]::NewLine + 'shell.Run ""powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File """"{app}\launch-app.ps1"""""" , 0, False')"""; Flags: runhidden waituntilterminated
Filename: "C:\Windows\System32\wscript.exe"; Parameters: """{app}\launch-app.vbs"""; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "taskkill.exe"; Parameters: "/F /IM pulsefeed.exe"; Flags: runhidden; RunOnceId: "StopPulseFeed"
