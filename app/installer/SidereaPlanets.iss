; Inno Setup script of the Siderea Planets installer (RK-026), modelled on the one of Siderea (siderea/windows/installer/Siderea.iss).
; Built by scripts/publish-windows.sh --installer, which passes
;   /DAppVersion=<version of app/package.json>  /DSourceDir=<build\windows\dist\Siderea Planets>  /DOutputDir=<build\windows\dist>
;   /DNotesEn=<notes-en.txt> /DNotesDe=<notes-de.txt>  /DWebView2Setup=<MicrosoftEdgeWebview2Setup.exe>
; Installs for the current user (no administrator rights) or, when asked, for all users. Siderea Planets needs the Microsoft Edge
; WebView2 runtime, which every Windows 11 and every updated Windows 10 has; where it is missing, Setup runs Microsoft's small
; bootstrapper, which downloads it. Uninstalling removes the program folder, not the settings and no project or recording.
; Unsigned: no certificate, so SmartScreen warns on a download.

#ifndef AppVersion
  #error AppVersion is not defined; build with scripts/publish-windows.sh --installer
#endif
#ifndef SourceDir
  #error SourceDir is not defined; build with scripts/publish-windows.sh --installer
#endif
#ifndef NotesEn
  #error NotesEn is not defined; build with scripts/publish-windows.sh --installer
#endif
#ifndef NotesDe
  #error NotesDe is not defined; build with scripts/publish-windows.sh --installer
#endif
#ifndef WebView2Setup
  #error WebView2Setup is not defined; build with scripts/publish-windows.sh --installer
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif

#define AppName "Siderea Planets"
#define AppExe "Siderea Planets.exe"

[Setup]
; Never change the AppId: it is how a newer setup finds the version it replaces.
AppId={{120B1A26-4EA3-4AB8-BF8A-07BC17678E5A}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=Philipp Kuhlmay
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
; Per user by default ({autopf} is then %LOCALAPPDATA%\Programs); the dialog offers "for all users", which asks for elevation.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
SetupIconFile=..\src-tauri\icons\icon.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
OutputDir={#OutputDir}
OutputBaseFilename=Siderea-Planets-{#AppVersion}-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Siderea Planets may be running during an upgrade; Setup closes it and starts the new version afterwards if asked to.
CloseApplications=yes
RestartApplications=no

[Languages]
; The "what is new" page of the wizard: the section of this version in app/CHANGELOG.md, written by publish-windows.sh.
Name: "english"; MessagesFile: "compiler:Default.isl"; InfoBeforeFile: "{#NotesEn}"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"; InfoBeforeFile: "{#NotesDe}"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[CustomMessages]
english.InstallingWebView2=Installing the Microsoft Edge WebView2 runtime that Siderea Planets needs…
german.InstallingWebView2=Die Microsoft-Edge-WebView2-Laufzeit, die Siderea Planets braucht, wird installiert …
english.WebView2Failed=The Microsoft Edge WebView2 runtime could not be installed. Siderea Planets needs it; it is available from microsoft.com.
german.WebView2Failed=Die Microsoft-Edge-WebView2-Laufzeit konnte nicht installiert werden. Siderea Planets braucht sie; es gibt sie bei microsoft.com.

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Microsoft's bootstrapper, only unpacked and run when the WebView2 runtime is missing (see [Code]).
Source: "{#WebView2Setup}"; DestDir: "{tmp}"; Flags: dontcopy

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[Code]
const
  WebView2Client = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';

{ The runtime counts as installed when EdgeUpdate lists it with a version, per machine (32- or 64-bit view) or per user. }
function WebView2Installed: Boolean;
var
  Version: String;
begin
  Result :=
    (RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\' + WebView2Client, 'pv', Version) and (Version <> '') and (Version <> '0.0.0.0')) or
    (RegQueryStringValue(HKLM, 'SOFTWARE\Microsoft\EdgeUpdate\Clients\' + WebView2Client, 'pv', Version) and (Version <> '') and (Version <> '0.0.0.0')) or
    (RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\' + WebView2Client, 'pv', Version) and (Version <> '') and (Version <> '0.0.0.0'));
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if (CurStep = ssPostInstall) and not WebView2Installed then
  begin
    WizardForm.StatusLabel.Caption := CustomMessage('InstallingWebView2');
    ExtractTemporaryFile('MicrosoftEdgeWebview2Setup.exe');
    if not Exec(ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe'), '/silent /install', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
      MsgBox(CustomMessage('WebView2Failed'), mbError, MB_OK);
  end;
end;
