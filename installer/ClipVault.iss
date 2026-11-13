; SPDX-License-Identifier: MIT
; ClipVault — Inno Setup 6 installer script
; Build with:
;   "C:\Users\Dodot\AppData\Local\Programs\Inno Setup 6\ISCC.exe" installer\ClipVault.iss
; Produces: installer\Output\ClipVault-0.1.3-Setup.exe
;
; Per-user install (no admin elevation): files go to {localappdata}\Programs\ClipVault,
; shortcuts to the current user's Start Menu. No HKLM writes, no services, no drivers.
; No network access at install time or runtime.

#define MyAppName "ClipVault"
#define MyAppVersion "0.1.3"
#define MyAppPublisher "Manuel"
#define MyAppURL "https://github.com/manuel/clipvault"
#define MyAppExeName "clipvault.exe"
#define MyAppCopyright "Copyright (C) 2026 Manuel — MIT"

[Setup]
AppId={{8F3A2C1E-7B4D-4E9A-9C6F-1D2E3A4B5C6D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
AppCopyright={#MyAppCopyright}
VersionInfoVersion={#MyAppVersion}.0
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppName} — private Windows clipboard manager
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}.0
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
DisableDirPage=no
AllowNoIcons=yes
; Per-user: no elevation required
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=ClipVault-0.1.3-Setup
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
WizardSizePercent=110
; Keep it calm and on-brand (light, minimal)
WizardImageFile=
WizardSmallImageFile=
DisableWelcomePage=no
LicenseFile=..\LICENSE
InfoBeforeFile=..\README.md
; No restart, no admin, no internet
RestartIfNeededByRun=no
ShowLanguageDialog=no
LanguageDetectionMethod=none
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
Uninstallable=yes
CreateUninstallRegKey=yes
UninstallDisplayName={#MyAppName} {#MyAppVersion}
; Do not touch the user's clipboard data on uninstall
UsedUserAreasWarning=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startupicon"; Description: "Launch ClipVault when Windows starts"; GroupDescription: "Startup:"; Flags: unchecked

[Files]
; Main executable (release build, LTO, stripped, embedded icon/version)
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Docs bundled for offline reference
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Registry]
; Optional launch-at-startup. Writes ONLY our own HKCU value; the app's own
; Settings toggle also writes this same value, so the two stay consistent.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; \
    ValueType: string; ValueName: "ClipVault"; ValueData: """{app}\{#MyAppExeName}"""; \
    Flags: uninsdeletevalue; Tasks: startupicon

[Run]
; Offer to launch after install (user can uncheck)
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; \
    Flags: nowait postinstall skipifsilent unchecked

[UninstallRun]
; Nothing to run. Tray icon is removed by the app's own NIM_DELETE on exit.

[UninstallDelete]
; Remove only files WE created in the install dir; leave %LOCALAPPDATA%\ClipVault
; (the database + settings + image assets) intact by default — it's user history.
Type: files; Name: "{app}\clipvault.log"

[Code]
// Quiet, on-brand note on the finishing page — no telemetry, no network.
procedure InitializeWizard();
begin
  WizardForm.WelcomeLabel1.Caption := 'Welcome to ClipVault';
  WizardForm.WelcomeLabel2.Caption :=
    'A calm, private Windows clipboard manager.' + #13#10 + #13#10 +
    'This installs ClipVault for your user account only. No administrator rights, ' +
    'no network access, no telemetry.' + #13#10 + #13#10 +
    'Your clipboard history is stored locally in %LOCALAPPDATA%\ClipVault and is ' +
    'NOT removed by the uninstaller.' + #13#10 + #13#10 +
    'Click Next to continue.';
end;

// The app's own named-mutex singleton prevents a second instance; Inno Setup
// natively detects a file-in-use condition and offers Retry/Ignore/Abort, so we
// do not force-kill a running instance (which would risk a WAL checkpoint).
