; CtrlLingo · 译点 Windows TSF installer. The DLLs are versioned because Windows
; keeps each one loaded in every host application until that application exits.
#define AppVersion "0.1.1"
#define AppName "CtrlLingo · 译点"
#define Publisher "boluomi2001"
#define WebsiteUrl "https://github.com/boluomi2001/ctrllingo"
#define Repo "..\..\.."
#ifndef PackageRoot
  #define PackageRoot "..\..\..\target\installer\.ctrllingo-package-stage"
#endif
#ifndef OutputDirectory
  #define OutputDirectory "..\..\..\target\installer"
#endif
#define PackageRootWithSlash AddBackslash(PackageRoot)
#define TsfDll "ctrllingo_tsf-" + AppVersion + ".dll"
#define TsfDll32 "ctrllingo_tsf-" + AppVersion + "-x86.dll"

[Setup]
AppId={{1C9BF7A3-19C8-45F0-BA38-E1D5B0CC62AA}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#Publisher}
AppSupportURL={#WebsiteUrl}
VersionInfoVersion={#AppVersion}
DefaultDirName={autopf}\CtrlLingo
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
PrivilegesRequired=admin
CloseApplications=no
OutputDir={#OutputDirectory}
OutputBaseFilename=CtrlLingo-{#AppVersion}-windows-x64-setup
SetupIconFile={#Repo}\apps\windows\tsf\resources\ctrllingo.ico
UninstallDisplayIcon={app}\ctrllingo.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "chs"; MessagesFile: "languages\ChineseSimplified.isl"

[Messages]
FinishedLabel=CtrlLingo 安装完成。请注销后重新登录，或重新打开要使用输入法的应用，使已打开的应用加载新版本。%n%n安装包未签名，候选窗在任务栏搜索和部分系统界面中可能被其他窗口遮挡。

[Files]
Source: "{#PackageRootWithSlash}payload\*"; DestDir: "{app}"; Excludes: "ctrllingo_tsf.dll,ctrllingo_tsf32.dll"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#PackageRootWithSlash}documentation\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Keep prior versions side by side: running host applications may still have these DLLs loaded.
Source: "{#PackageRootWithSlash}payload\ctrllingo_tsf.dll"; DestDir: "{app}"; DestName: "{#TsfDll}"; Flags: ignoreversion uninsrestartdelete
Source: "{#PackageRootWithSlash}payload\ctrllingo_tsf32.dll"; DestDir: "{app}"; DestName: "{#TsfDll32}"; Flags: ignoreversion uninsrestartdelete

[Icons]
Name: "{group}\CtrlLingo 设置"; Filename: "{app}\ctrllingo-settings.exe"; IconFilename: "{app}\ctrllingo.ico"
Name: "{group}\卸载 CtrlLingo"; Filename: "{uninstallexe}"
Name: "{commonstartup}\CtrlLingo Server"; Filename: "{app}\ctrllingo-server.exe"; WorkingDir: "{app}"; IconFilename: "{app}\ctrllingo.ico"

[Run]
; Elevated installation writes the starter config in the original user's profile.
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File ""{app}\initialize-user-config.ps1"" -DefaultConfigPath ""{app}\default-config.toml"""; Flags: runhidden waituntilterminated runasoriginaluser

[UninstallDelete]
Type: files; Name: "{app}\ctrllingo_tsf-*.dll"
Type: files; Name: "{app}\ctrllingo-server.old-*.exe"
Type: files; Name: "{commonappdata}\CtrlLingo\ctrllingo.ico"
Type: dirifempty; Name: "{commonappdata}\CtrlLingo"

[Code]
function CreateMutex(Attributes: Longint; InitialOwner: BOOL; Name: String): THandle;
  external 'CreateMutexW@kernel32.dll stdcall';

procedure HoldInstallerMutex;
begin
  if CreateMutex(0, False, 'Global\CtrlLingoInstaller') = 0 then
    RaiseException('无法创建 CtrlLingo 安装互斥体。');
end;

function InitializeSetup: Boolean;
begin
  HoldInstallerMutex;
  Result := True;
end;

function InitializeUninstall: Boolean;
begin
  HoldInstallerMutex;
  Result := True;
end;

procedure RunChecked(const FileName, Parameters: String; AllowMissingProcess: Boolean);
var
  ResultCode: Integer;
begin
  if not Exec(FileName, Parameters, '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
    RaiseException('无法启动 ' + FileName + '。');
  if (ResultCode <> 0) and not (AllowMissingProcess and (ResultCode = 128)) then
    RaiseException(FileName + ' 失败，退出码 ' + IntToStr(ResultCode) + '。');
end;

procedure KillCtrlLingoProcess(const Image: String);
begin
  RunChecked(ExpandConstant('{sys}\taskkill.exe'), '/im ' + Image + ' /t /f', True);
end;

procedure RetireServer;
var
  Path, Retired: String;
begin
  Path := ExpandConstant('{app}\ctrllingo-server.exe');
  if FileExists(Path) then
  begin
    Retired := ExpandConstant('{app}\ctrllingo-server.old-') + IntToStr(Random(1000000)) + '.exe';
    if not RenameFile(Path, Retired) then
      RaiseException('改名正在运行的 CtrlLingo Server 失败: ' + Path);
  end;
end;

procedure RetireLoadedDll(const Name: String);
var
  Path, Retired: String;
begin
  Path := ExpandConstant('{app}\') + Name;
  if FileExists(Path) then
  begin
    Retired := ExpandConstant('{app}\ctrllingo_tsf-') + IntToStr(Random(1000000)) + '.old.dll';
    if not RenameFile(Path, Retired) then
      RaiseException('改名已加载的 CtrlLingo TSF DLL 失败: ' + Path);
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  RetireServer;
  KillCtrlLingoProcess('ctrllingo-server.exe');
  KillCtrlLingoProcess('ctrllingo-settings.exe');
  RetireLoadedDll('{#TsfDll}');
  RetireLoadedDll('{#TsfDll32}');
  Result := '';
end;

procedure DeleteLegacyTask;
var
  ResultCode: Integer;
begin
  if not Exec(ExpandConstant('{sys}\schtasks.exe'), '/delete /tn "CtrlLingo Server" /f', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
    RaiseException('无法启动 schtasks.exe。');
  if (ResultCode <> 0) and (ResultCode <> 1) then
    RaiseException('删除 CtrlLingo Server 旧计划任务失败，退出码 ' + IntToStr(ResultCode) + '。');
end;

procedure RemoveStaleDlls;
var
  Dir, Path: String;
  Found: TFindRec;
begin
  Dir := ExpandConstant('{app}');
  if FindFirst(Dir + '\ctrllingo_tsf-*.dll', Found) then
  begin
    try
      repeat
        if (CompareText(Found.Name, '{#TsfDll}') <> 0) and (CompareText(Found.Name, '{#TsfDll32}') <> 0) then
        begin
          Path := Dir + '\' + Found.Name;
          if not DeleteFile(Path) then
            RestartReplace(Path, '');
        end;
      until not FindNext(Found);
    finally
      FindClose(Found);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  SysDir, AppDir: String;
begin
  if CurStep = ssPostInstall then
  begin
    SysDir := ExpandConstant('{sys}');
    AppDir := ExpandConstant('{app}');
    RunChecked(ExpandConstant('{sys}\icacls.exe'), '"' + AppDir + '" /grant *S-1-15-2-1:(OI)(CI)RX /T /C /Q', False);
    RunChecked(SysDir + '\regsvr32.exe', '/s "' + AppDir + '\{#TsfDll}"', False);
    RunChecked(ExpandConstant('{syswow64}\regsvr32.exe'), '/s "' + AppDir + '\{#TsfDll32}"', False);
    DeleteLegacyTask;
    RemoveStaleDlls;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  SysDir, AppDir: String;
begin
  if CurUninstallStep = usUninstall then
  begin
    RetireServer;
    KillCtrlLingoProcess('ctrllingo-server.exe');
    KillCtrlLingoProcess('ctrllingo-settings.exe');
    SysDir := ExpandConstant('{sys}');
    AppDir := ExpandConstant('{app}');
    if FileExists(AppDir + '\{#TsfDll}') then
      RunChecked(SysDir + '\regsvr32.exe', '/u /s "' + AppDir + '\{#TsfDll}"', False);
    if FileExists(AppDir + '\{#TsfDll32}') then
      RunChecked(ExpandConstant('{syswow64}\regsvr32.exe'), '/u /s "' + AppDir + '\{#TsfDll32}"', False);
    DeleteLegacyTask;
  end;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  ErrorCode: Integer;
begin
  Result := True;
  if (CurPageID = wpFinished) and (not WizardSilent) then
    ShellExecAsOriginalUser('', ExpandConstant('{app}\ctrllingo-server.exe'), '', ExpandConstant('{app}'), SW_SHOWNORMAL, ewNoWait, ErrorCode);
end;
