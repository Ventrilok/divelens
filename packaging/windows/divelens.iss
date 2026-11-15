; Dive Lens installer for Windows (Inno Setup 6).
;
; Installs for the current user only (no administrator rights) the plug-in
; folder DiveLens.lrplugin, with the app inside, into the user's Lightroom
; Modules folder, %APPDATA%\Adobe\Lightroom\Modules, where Lightroom loads it
; by itself. The app's own updates can then replace it without asking for
; rights.
;
; Built by the release workflow:
;   ISCC /DVersion=2.0.0 /DNumericVersion=2.0.0 /DPluginDir=<DiveLens.lrplugin>
;        /DOutputDir=<folder> packaging\windows\divelens.iss
;
; Part of Dive Lens, GPL-3.0-or-later.

#ifndef Version
  #error Version is required (/DVersion=...)
#endif
#ifndef NumericVersion
  #error NumericVersion is required (/DNumericVersion=...)
#endif
#ifndef PluginDir
  #error PluginDir is required (/DPluginDir=...)
#endif
#ifndef OutputDir
  #define OutputDir "."
#endif

[Setup]
; Never change AppId: Windows recognises an installed Dive Lens by it.
AppId={{07FDD4D5-BDA4-417F-BABA-900F10319ACF}
AppName=Dive Lens
AppVersion={#Version}
AppVerName=Dive Lens {#Version}
AppPublisher=Pierre-Benoît Querton
AppPublisherURL=https://github.com/Ventrilok/divelens
VersionInfoVersion={#NumericVersion}
VersionInfoProductTextVersion={#Version}
PrivilegesRequired=lowest
DefaultDirName={userappdata}\Adobe\Lightroom\Modules\DiveLens.lrplugin
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
; The uninstaller stays out of the plug-in folder, which updates replace whole.
UninstallFilesDir={localappdata}\Dive Lens\Uninstall
UninstallDisplayName=Dive Lens
UninstallDisplayIcon={app}\Dive Lens.exe
SetupIconFile=..\..\app\src-tauri\icons\icon.ico
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
OutputDir={#OutputDir}
OutputBaseFilename=DiveLens-{#Version}-Windows-Setup

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"

[InstallDelete]
; Start from an empty plug-in folder: no file of an earlier version is left.
Type: filesandordirs; Name: "{app}\*"

[Files]
Source: "{#PluginDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
