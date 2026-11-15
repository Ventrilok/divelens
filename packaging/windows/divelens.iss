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
; Saved as UTF-8 with a byte-order mark, so that "Pierre-Benoît" reads right.
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
; English or French, following Windows, without asking.
ShowLanguageDialog=auto
Compression=lzma2
SolidCompression=yes
OutputDir={#OutputDir}
OutputBaseFilename=DiveLens-{#Version}-Windows-Setup

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"

[Messages]
; Last screen. No shortcut is created, so Inno Setup shows the "NoIcons" text.
en.FinishedLabelNoIcons=Dive Lens is installed.%n%nIf Lightroom Classic is open, quit it and start it again: Lightroom loads Dive Lens only when it starts.%n%nThe Dive Lens menus are in Library › Plug-in Extras.
en.FinishedLabel=Dive Lens is installed.%n%nIf Lightroom Classic is open, quit it and start it again: Lightroom loads Dive Lens only when it starts.%n%nThe Dive Lens menus are in Library › Plug-in Extras.
fr.FinishedLabelNoIcons=Dive Lens est installé.%n%nSi Lightroom Classic est ouvert, quittez-le et relancez-le : Lightroom ne charge Dive Lens qu'à son démarrage.%n%nLes menus Dive Lens sont dans Bibliothèque › Modules externes.
fr.FinishedLabel=Dive Lens est installé.%n%nSi Lightroom Classic est ouvert, quittez-le et relancez-le : Lightroom ne charge Dive Lens qu'à son démarrage.%n%nLes menus Dive Lens sont dans Bibliothèque › Modules externes.

[InstallDelete]
; Start from an empty plug-in folder: no file of an earlier version is left.
Type: filesandordirs; Name: "{app}\*"

[Files]
Source: "{#PluginDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Run]
; An update made by the app (spec F13) runs this installer silently with
; /update=1, then quits: start the new version, on Settings. A first
; installation starts nothing.
Filename: "{app}\Dive Lens.exe"; Parameters: "--after-update --screen settings"; Flags: nowait; Check: IsUpdate

[Code]
function IsUpdate: Boolean;
begin
  Result := ExpandConstant('{param:update|0}') = '1';
end;
