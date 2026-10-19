param(
    [string]$OriginalAppData,
    [string]$OriginalStartup
)

$ErrorActionPreference = 'Stop'
$scriptPath = [System.IO.Path]::GetFullPath($MyInvocation.MyCommand.Path)
$logPath = Join-Path $PSScriptRoot 'install-test.log'
$transcriptStarted = $false
$exitCode = 0
$installerGate = $null

function Start-InstallTranscript {
    Start-Transcript -Path $logPath -Append -Force | Out-Null
    $script:transcriptStarted = $true
}

function Stop-InstallTranscript {
    if ($script:transcriptStarted) {
        Stop-Transcript | Out-Null
        $script:transcriptStarted = $false
    }
}

function ConvertTo-CommandLineArgument {
    param([string]$Value)

    $builder = New-Object System.Text.StringBuilder
    [void]$builder.Append([char]34)
    $slashes = 0
    foreach ($character in $Value.ToCharArray()) {
        if ([int]$character -eq 92) {
            $slashes++
            continue
        }
        if ([int]$character -eq 34) {
            for ($i = 0; $i -lt (2 * $slashes + 1); $i++) { [void]$builder.Append([char]92) }
            [void]$builder.Append([char]34)
            $slashes = 0
            continue
        }
        for ($i = 0; $i -lt $slashes; $i++) { [void]$builder.Append([char]92) }
        [void]$builder.Append($character)
        $slashes = 0
    }
    for ($i = 0; $i -lt (2 * $slashes); $i++) { [void]$builder.Append([char]92) }
    [void]$builder.Append([char]34)
    $builder.ToString()
}

function Get-CurrentPowerShellPath {
    if ([Environment]::Is64BitProcess) {
        return Join-Path $env:WINDIR 'System32\WindowsPowerShell\v1.0\powershell.exe'
    }
    return Join-Path $env:WINDIR 'Sysnative\WindowsPowerShell\v1.0\powershell.exe'
}

function Start-InstallerCopy {
    param([switch]$RunAs)

    $arguments = @(
        '-NoProfile',
        '-ExecutionPolicy', 'Bypass',
        '-File', (ConvertTo-CommandLineArgument $scriptPath),
        '-OriginalAppData', (ConvertTo-CommandLineArgument $OriginalAppData),
        '-OriginalStartup', (ConvertTo-CommandLineArgument $OriginalStartup)
    ) -join ' '
    $parameters = @{
        FilePath = (Get-CurrentPowerShellPath)
        ArgumentList = $arguments
        WindowStyle = 'Hidden'
        Wait = $true
        PassThru = $true
    }
    if ($RunAs) { $parameters.Verb = 'RunAs' }
    Start-Process @parameters
}

function Get-PeMachine {
    param([string]$Path)

    $stream = [System.IO.File]::OpenRead($Path)
    $reader = New-Object System.IO.BinaryReader($stream)
    try {
        if ($reader.ReadUInt16() -ne 0x5A4D) { throw "不是有效的 Windows 可执行文件：$Path" }
        $stream.Position = 0x3C
        $peOffset = $reader.ReadInt32()
        if ($peOffset -lt 0x40 -or $peOffset -gt ($stream.Length - 6)) { throw "可执行文件头无效：$Path" }
        $stream.Position = $peOffset
        if ($reader.ReadUInt32() -ne 0x00004550) { throw "不是有效的 Windows PE 文件：$Path" }
        return $reader.ReadUInt16()
    }
    finally {
        $reader.Dispose()
        $stream.Dispose()
    }
}

try {
    Start-InstallTranscript
    if (-not $OriginalAppData) { $OriginalAppData = $env:APPDATA }
    if (-not $OriginalStartup) { $OriginalStartup = Join-Path $OriginalAppData 'Microsoft\Windows\Start Menu\Programs\Startup' }
    if (-not [System.IO.Path]::IsPathRooted($OriginalAppData) -or -not [System.IO.Path]::IsPathRooted($OriginalStartup)) {
        throw '无法确定启动者的绝对 APPDATA 或 Startup 路径。'
    }
    $OriginalAppData = [System.IO.Path]::GetFullPath($OriginalAppData)
    $OriginalStartup = [System.IO.Path]::GetFullPath($OriginalStartup)

    if (-not [Environment]::Is64BitOperatingSystem) { throw '此测试版只支持 64 位 Windows。' }
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    $isAdministrator = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if (-not $isAdministrator -or -not [Environment]::Is64BitProcess) {
        $needElevation = -not $isAdministrator
        Stop-InstallTranscript
        $child = Start-InstallerCopy -RunAs:$needElevation
        Start-InstallTranscript
        if ($child.ExitCode -ne 0) { throw "提升后的安装脚本失败，退出码：$($child.ExitCode)" }
        Write-Output '提升后的安装脚本已成功结束。'
        return
    }

    $installerGate = New-Object System.Threading.Mutex($false, 'Global\QingjianBilingualTestInstaller')
    $sourceRoot = Join-Path $PSScriptRoot 'app'
    $targetRoot = Join-Path $env:ProgramFiles 'QingjianBilingualTest'
    $testDll = Join-Path $targetRoot 'qingjian_tsf.dll'
    $defaultConfig = Join-Path $PSScriptRoot 'default-config.toml'
    $requiredFiles = @(
        'qingjian_tsf.dll',
        'qingjian-server.exe',
        'qingjian-settings.exe',
        'data\generated\dict.qj',
        'data\generated\glossary-en.qj'
    )

    foreach ($relativePath in $requiredFiles) {
        $file = Join-Path $sourceRoot $relativePath
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "测试包缺少必需文件：$file" }
    }
    if (-not (Test-Path -LiteralPath $defaultConfig -PathType Leaf)) { throw "测试包缺少默认配置：$defaultConfig" }
    foreach ($relativePath in @('qingjian_tsf.dll', 'qingjian-server.exe', 'qingjian-settings.exe')) {
        $file = Join-Path $sourceRoot $relativePath
        if ((Get-PeMachine $file) -ne 0x8664) { throw "必需文件不是 64 位程序：$file" }
    }
    $sourceDll32 = Join-Path $sourceRoot 'qingjian_tsf32.dll'
    if ((Test-Path -LiteralPath $sourceDll32 -PathType Leaf) -and (Get-PeMachine $sourceDll32) -ne 0x014C) {
        throw "32 位 TSF DLL 架构不匹配：$sourceDll32"
    }

    if (Test-Path -LiteralPath $targetRoot) {
        if (Test-Path -LiteralPath $testDll -PathType Leaf) {
            throw "测试版目录已有自己的 DLL，不能覆盖可能仍被加载的 DLL。请先运行卸载测试版、重启 Windows，再手动移除目录：$targetRoot"
        }
        throw "目标目录已存在但未找到测试版 DLL，拒绝覆盖未知内容：$targetRoot"
    }

    New-Item -ItemType Directory -Path $targetRoot | Out-Null
    Get-ChildItem -LiteralPath $sourceRoot -Force | Copy-Item -Destination $targetRoot -Recurse -Force

    $regsvr64 = Join-Path $env:WINDIR 'System32\regsvr32.exe'
    $regsvr32 = Join-Path $env:WINDIR 'SysWOW64\regsvr32.exe'
    if (-not (Test-Path -LiteralPath $regsvr64 -PathType Leaf)) { throw "找不到 64 位 regsvr32：$regsvr64" }
    $registrationFailures = New-Object 'System.Collections.Generic.List[string]'
    & $regsvr64 /s $testDll
    $registrationExit = $LASTEXITCODE
    if ($registrationExit -ne 0) { $registrationFailures.Add("64 位 DLL 注册失败，退出码 $registrationExit：$testDll") }
    else { Write-Output "64 位 TSF DLL 注册成功：$testDll" }

    $targetDll32 = Join-Path $targetRoot 'qingjian_tsf32.dll'
    if (Test-Path -LiteralPath $targetDll32 -PathType Leaf) {
        if (-not (Test-Path -LiteralPath $regsvr32 -PathType Leaf)) {
            $registrationFailures.Add("找不到 32 位 regsvr32：$regsvr32")
        }
        else {
            & $regsvr32 /s $targetDll32
            $registrationExit = $LASTEXITCODE
            if ($registrationExit -ne 0) { $registrationFailures.Add("32 位 DLL 注册失败，退出码 $registrationExit：$targetDll32") }
            else { Write-Output "32 位 TSF DLL 注册成功：$targetDll32" }
        }
    }
    if ($registrationFailures.Count -gt 0) { throw ($registrationFailures -join [Environment]::NewLine) }

    $userConfigDir = Join-Path $OriginalAppData 'QingjianBilingualTest'
    $userConfig = Join-Path $userConfigDir 'config.toml'
    if (-not (Test-Path -LiteralPath $userConfig)) {
        if (-not (Test-Path -LiteralPath $userConfigDir)) { New-Item -ItemType Directory -Path $userConfigDir | Out-Null }
        Copy-Item -LiteralPath $defaultConfig -Destination $userConfig
        Write-Output "已创建测试版用户配置：$userConfig"
    }
    else {
        Write-Output "保留已存在的测试版用户配置：$userConfig"
    }

    if (-not (Test-Path -LiteralPath $OriginalStartup)) { New-Item -ItemType Directory -Path $OriginalStartup | Out-Null }
    $shortcutPath = Join-Path $OriginalStartup 'QingjianBilingualTest Server.lnk'
    $shell = New-Object -ComObject WScript.Shell
    try {
        $shortcut = $shell.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = Join-Path $targetRoot 'qingjian-server.exe'
        $shortcut.WorkingDirectory = $targetRoot
        $shortcut.Save()
    }
    finally {
        [void][Runtime.InteropServices.Marshal]::ReleaseComObject($shell)
    }

    Write-Output '测试版安装完成。请注销并重新登录，再按 Win + Space 选择“青简双语测试”。Server 不会在管理员进程中启动；启动快捷方式会在下次登录时由当前用户运行。'
}
catch {
    if (-not $transcriptStarted) {
        try { Start-InstallTranscript } catch { }
    }
    Write-Output "安装失败：$($_.Exception.Message)"
    $exitCode = 1
}
finally {
    if ($installerGate) { $installerGate.Dispose() }
    Stop-InstallTranscript
}

exit $exitCode
