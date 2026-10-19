param(
    [string]$OriginalAppData,
    [string]$OriginalStartup
)

$ErrorActionPreference = 'Stop'
$scriptPath = [System.IO.Path]::GetFullPath($MyInvocation.MyCommand.Path)
$logPath = Join-Path $PSScriptRoot 'uninstall-test.log'
$transcriptStarted = $false
$exitCode = 0
$installerGate = $null

function Start-UninstallTranscript {
    Start-Transcript -Path $logPath -Append -Force | Out-Null
    $script:transcriptStarted = $true
}

function Stop-UninstallTranscript {
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

function Start-UninstallerCopy {
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

function Invoke-RegSvr32 {
    param(
        [string]$Executable,
        [string]$DllPath,
        [string]$Architecture
    )

    if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
        throw "找不到 $Architecture regsvr32：$Executable"
    }
    & $Executable /u /s $DllPath
    $registrationExit = $LASTEXITCODE
    if ($registrationExit -ne 0) {
        throw "$Architecture TSF DLL 反注册失败，退出码 $registrationExit：$DllPath"
    }
    Write-Output "$Architecture TSF DLL 反注册成功：$DllPath"
}

try {
    Start-UninstallTranscript
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
        Stop-UninstallTranscript
        $child = Start-UninstallerCopy -RunAs:$needElevation
        Start-UninstallTranscript
        if ($child.ExitCode -ne 0) { throw "提升后的卸载脚本失败，退出码：$($child.ExitCode)" }
        Write-Output '提升后的卸载脚本已成功结束。'
        return
    }

    $installerGate = New-Object System.Threading.Mutex($false, 'Global\QingjianBilingualTestInstaller')
    $targetRoot = Join-Path $env:ProgramFiles 'QingjianBilingualTest'
    $testDll = Join-Path $targetRoot 'qingjian_tsf.dll'
    if (-not (Test-Path -LiteralPath $testDll -PathType Leaf)) {
        Write-Output "未找到测试版自己的 DLL，不执行卸载：$testDll"
        return
    }

    $rootPrefix = [System.IO.Path]::GetFullPath($targetRoot).TrimEnd('\') + '\'
    $processes = Get-CimInstance -ClassName Win32_Process -Filter "Name = 'qingjian-server.exe' OR Name = 'qingjian-settings.exe'"
    foreach ($process in $processes) {
        $executablePath = $process.ExecutablePath
        if (-not $executablePath -or -not [System.IO.Path]::IsPathRooted($executablePath)) { continue }
        $fullExecutablePath = [System.IO.Path]::GetFullPath($executablePath)
        $executableName = [System.IO.Path]::GetFileName($fullExecutablePath)
        if (($executableName -ieq 'qingjian-server.exe' -or $executableName -ieq 'qingjian-settings.exe') -and
            $fullExecutablePath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            Stop-Process -Id ([int]$process.ProcessId) -Force
            Write-Output "已停止测试版进程：$fullExecutablePath"
        }
    }

    $regsvr64 = Join-Path $env:WINDIR 'System32\regsvr32.exe'
    $regsvr32 = Join-Path $env:WINDIR 'SysWOW64\regsvr32.exe'
    $unregistrationFailures = New-Object 'System.Collections.Generic.List[string]'
    try { Invoke-RegSvr32 -Executable $regsvr64 -DllPath $testDll -Architecture '64 位' }
    catch { $unregistrationFailures.Add($_.Exception.Message) }

    $testDll32 = Join-Path $targetRoot 'qingjian_tsf32.dll'
    if (Test-Path -LiteralPath $testDll32 -PathType Leaf) {
        try { Invoke-RegSvr32 -Executable $regsvr32 -DllPath $testDll32 -Architecture '32 位' }
        catch { $unregistrationFailures.Add($_.Exception.Message) }
    }

    $shortcutPath = Join-Path $OriginalStartup 'QingjianBilingualTest Server.lnk'
    if (Test-Path -LiteralPath $shortcutPath -PathType Leaf) {
        Remove-Item -LiteralPath $shortcutPath -Force
        Write-Output "已删除测试版启动快捷方式：$shortcutPath"
    }

    if ($unregistrationFailures.Count -gt 0) { throw ($unregistrationFailures -join [Environment]::NewLine) }
    Write-Output "测试版 DLL 已反注册，用户配置保留。重启 Windows 后可手动删除安装目录：$targetRoot"
}
catch {
    if (-not $transcriptStarted) {
        try { Start-UninstallTranscript } catch { }
    }
    Write-Output "卸载失败：$($_.Exception.Message)"
    $exitCode = 1
}
finally {
    if ($installerGate) { $installerGate.Dispose() }
    Stop-UninstallTranscript
}

exit $exitCode
