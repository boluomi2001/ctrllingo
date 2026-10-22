[CmdletBinding()]
param(
    [string]$Target = 'x86_64-pc-windows-msvc',
    [string]$Target32 = 'i686-pc-windows-msvc',
    [string]$BinaryDirectory,
    [string]$BinaryDirectory32,
    [switch]$SkipBuild,
    [string]$OutputDirectory = 'target\ctrllingo-release'
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$iss = Join-Path $PSScriptRoot 'ctrllingo.iss'
$previousUiAccess = $env:QINGJIAN_UIACCESS
$env:QINGJIAN_UIACCESS = '0'

function Invoke-CheckedNative {
    param([string]$FilePath, [string[]]$ArgumentList, [string]$Description)
    & $FilePath @ArgumentList
    if ($LASTEXITCODE -ne 0) { throw "$Description 失败，退出码 $LASTEXITCODE" }
}

function Copy-RequiredFile {
    param([string]$Source, [string]$Destination)
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少必需文件：$Source" }
    $parent = Split-Path -Parent $Destination
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

function Add-HashLines {
    param([string]$Root, [string]$Destination)
    $lines = Get-ChildItem -LiteralPath $Root -File -Recurse | Where-Object { $_.FullName -ne $Destination } | Sort-Object FullName | ForEach-Object {
        $relative = $_.FullName.Substring($Root.Length + 1).Replace('\', '/')
        "{0}  {1}" -f (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $relative
    }
    $lines | Set-Content -LiteralPath $Destination -Encoding UTF8
}

function Assert-UnderTarget {
    param([string]$Path)
    $targetRoot = [System.IO.Path]::GetFullPath((Join-Path $repo 'target')).TrimEnd('\')
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $targetPrefix = $targetRoot + '\'
    if (-not $fullPath.StartsWith($targetPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "只允许清理或写入仓库 target 目录内的打包路径：$fullPath"
    }
    return $fullPath
}

function Get-ReachablePackages {
    param($Metadata, [string[]]$RootNames)
    if (-not $Metadata.resolve) { throw 'cargo metadata 未返回依赖图。' }
    $packagesById = @{}
    foreach ($package in $Metadata.packages) { $packagesById[$package.id] = $package }
    $nodesById = @{}
    foreach ($node in $Metadata.resolve.nodes) { $nodesById[$node.id] = $node }

    $pending = New-Object System.Collections.ArrayList
    foreach ($package in $Metadata.packages | Where-Object { $_.name -in $RootNames }) { [void]$pending.Add($package.id) }
    if ($pending.Count -eq 0) { throw "cargo metadata 中找不到根依赖：$($RootNames -join ', ')" }
    $visited = @{}
    $reachable = @()
    while ($pending.Count -gt 0) {
        $last = $pending.Count - 1
        $id = $pending[$last]
        $pending.RemoveAt($last)
        if ($visited.ContainsKey($id)) { continue }
        $visited[$id] = $true
        if ($packagesById.ContainsKey($id)) { $reachable += $packagesById[$id] }
        if ($nodesById.ContainsKey($id)) {
            foreach ($dependency in $nodesById[$id].deps) { [void]$pending.Add($dependency.pkg) }
        }
    }
    return $reachable
}

try {
    $outputRootCandidate = if ([System.IO.Path]::IsPathRooted($OutputDirectory)) { $OutputDirectory } else { Join-Path $repo $OutputDirectory }
    $outputRoot = Assert-UnderTarget $outputRootCandidate
    Push-Location $repo
    try {
        if (-not $SkipBuild) {
            Invoke-CheckedNative 'cargo' @('build', '--release', '--locked', '--target', $Target, '-p', 'qingjian-windows-server', '-p', 'qingjian-windows-tsf', '-p', 'qingjian-settings') '64 位 MSVC 构建'
            Invoke-CheckedNative 'cargo' @('build', '--release', '--locked', '--target', $Target32, '-p', 'qingjian-windows-tsf') '32 位 MSVC TSF 构建'
        }
    } finally { Pop-Location }

    $binaryRoot = if ($BinaryDirectory) { (Resolve-Path -LiteralPath $BinaryDirectory).Path } else { Join-Path $repo "target\$Target\release" }
    $binaryRoot32 = if ($BinaryDirectory32) { (Resolve-Path -LiteralPath $BinaryDirectory32).Path } else { Join-Path $repo "target\$Target32\release" }
    foreach ($name in @('ctrllingo-server.exe', 'ctrllingo-settings.exe', 'qingjian_tsf.dll')) {
        if (-not (Test-Path -LiteralPath (Join-Path $binaryRoot $name) -PathType Leaf)) { throw "缺少 64 位产物：$(Join-Path $binaryRoot $name)" }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $binaryRoot32 'qingjian_tsf.dll') -PathType Leaf)) { throw "缺少 32 位 TSF 产物：$(Join-Path $binaryRoot32 'qingjian_tsf.dll')" }

    $fetchScript = Join-Path $repo 'tools\fetch-data.ps1'
    Invoke-CheckedNative 'powershell.exe' @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $fetchScript) '按 data.lock 获取并校验产品数据'
    $productData = Join-Path $repo 'target\release-data\unpacked\data'
    foreach ($relative in @(
        'generated\dict.qj', 'generated\lm.qj', 'generated\glossary-en.qj', 'generated\glossary-ja.qj',
        'generated\glossary-zh.qj', 'generated\glossary-es.qj', 'generated\glossary-vi.qj', 'generated\glossary-de.qj',
        'generated\english.tsv', 'generated\english-frequency.tsv',
        'models\hanzhang-zhiwei\hanzhang-zhiwei-small.qjm', 'models\hanzhang-tongbian\hanzhang-tongbian-small.qjm'
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $productData $relative) -PathType Leaf)) { throw "data-v7 包缺少 data\$relative" }
    }
    $domainDictionaries = @(Get-ChildItem -LiteralPath (Join-Path $productData 'generated\dicts') -Filter '*.qj' -File -ErrorAction SilentlyContinue)
    $codeTables = @(Get-ChildItem -LiteralPath (Join-Path $productData 'generated\codes') -Filter '*.qj' -File -ErrorAction SilentlyContinue)
    if ($domainDictionaries.Count -eq 0) { throw 'data-v7 包缺少 data/generated/dicts/*.qj' }
    if ($codeTables.Count -eq 0) { throw 'data-v7 包缺少 data/generated/codes/*.qj' }

    New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
    $stage = Assert-UnderTarget (Join-Path $outputRoot '.ctrllingo-package-stage')
    if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    $payload = Join-Path $stage 'payload'
    $documentation = Join-Path $stage 'documentation'
    New-Item -ItemType Directory -Path $payload, $documentation -Force | Out-Null

    Copy-RequiredFile (Join-Path $binaryRoot 'ctrllingo-server.exe') (Join-Path $payload 'ctrllingo-server.exe')
    Copy-RequiredFile (Join-Path $binaryRoot 'ctrllingo-settings.exe') (Join-Path $payload 'ctrllingo-settings.exe')
    Copy-RequiredFile (Join-Path $binaryRoot 'qingjian_tsf.dll') (Join-Path $payload 'ctrllingo_tsf.dll')
    Copy-RequiredFile (Join-Path $binaryRoot32 'qingjian_tsf.dll') (Join-Path $payload 'ctrllingo_tsf32.dll')
    Copy-RequiredFile (Join-Path $repo 'apps\windows\tsf\resources\ctrllingo.ico') (Join-Path $payload 'ctrllingo.ico')
    Copy-RequiredFile (Join-Path $PSScriptRoot 'default-config.toml') (Join-Path $payload 'default-config.toml')
    @'
param([Parameter(Mandatory=$true)][string]$DefaultConfigPath)
$ErrorActionPreference = 'Stop'
$directory = Join-Path $env:APPDATA 'CtrlLingo'
New-Item -ItemType Directory -Path $directory -Force | Out-Null
$config = Join-Path $directory 'config.toml'
if (-not (Test-Path -LiteralPath $config)) {
    Copy-Item -LiteralPath $DefaultConfigPath -Destination $config
}
'@ | Set-Content -LiteralPath (Join-Path $payload 'initialize-user-config.ps1') -Encoding UTF8

    $dataDestination = Join-Path $payload 'data'
    Copy-RequiredFile (Join-Path $productData 'generated\dict.qj') (Join-Path $dataDestination 'generated\dict.qj')
    Copy-RequiredFile (Join-Path $productData 'generated\lm.qj') (Join-Path $dataDestination 'generated\lm.qj')
    foreach ($file in Get-ChildItem -LiteralPath (Join-Path $productData 'generated') -File | Where-Object { $_.Extension -in @('.qj', '.tsv') }) {
        Copy-RequiredFile $file.FullName (Join-Path $dataDestination (Join-Path 'generated' $file.Name))
    }
    foreach ($folder in @('dicts', 'codes')) {
        $sourceDir = Join-Path $productData "generated\$folder"
        foreach ($file in Get-ChildItem -LiteralPath $sourceDir -Filter '*.qj' -File) {
            Copy-RequiredFile $file.FullName (Join-Path $dataDestination (Join-Path "generated\$folder" $file.Name))
        }
    }
    foreach ($model in @(
        'models\hanzhang-zhiwei\hanzhang-zhiwei-small.qjm',
        'models\hanzhang-tongbian\hanzhang-tongbian-small.qjm'
    )) { Copy-RequiredFile (Join-Path $productData $model) (Join-Path $dataDestination $model) }

    $assetFiles = @(
        'emoji\emoji-zh.tsv', 'emoji\emoji-en.tsv', 'emoji\LICENSE-unicode.txt',
        'levels\levels-en.tsv', 'levels\levels-ja.tsv', 'levels\README.md',
        'wubi\wubi86.tsv', 'wubi\README.md', 'wubi\LICENSE',
        'sample\dict.tsv', 'sample\english.tsv', 'sample\glossary-en.tsv', 'sample\glossary-ja.tsv',
        'sample\glossary-vi.tsv', 'sample\glossary-de.tsv', 'sample\README.md',
        'glossary\glossary-en.tsv', 'glossary\glossary-ja.tsv', 'glossary\glossary-zh.tsv',
        'glossary\glossary-es.tsv', 'glossary\glossary-vi.tsv', 'glossary\glossary-de.tsv', 'glossary\README.md',
        'stroke\README.md', 'stroke\LICENSE-CNS11643.txt'
    )
    foreach ($relative in $assetFiles) {
        Copy-RequiredFile (Join-Path $repo "assets\$relative") (Join-Path $payload (Join-Path 'assets' $relative))
    }

    foreach ($relative in @('LICENSE', 'NOTICE', 'README.md', 'SECURITY.md', 'SUPPORT.md')) {
        Copy-RequiredFile (Join-Path $repo $relative) (Join-Path $documentation (Split-Path -Leaf $relative))
    }
    Copy-RequiredFile (Join-Path $repo 'docs\PRIVACY.md') (Join-Path $documentation 'docs\PRIVACY.md')
    Copy-RequiredFile (Join-Path $repo 'docs\RELEASE-VALIDATION.md') (Join-Path $documentation 'docs\RELEASE-VALIDATION.md')
    Copy-RequiredFile (Join-Path $repo 'docs\KNOWN-ISSUES.md') (Join-Path $documentation 'docs\KNOWN-ISSUES.md')
    Copy-RequiredFile (Join-Path $repo 'docs\UPSTREAM.md') (Join-Path $documentation 'docs\UPSTREAM.md')
    $assetNotices = Get-ChildItem -LiteralPath (Join-Path $repo 'assets') -File -Recurse | Where-Object { $_.Name -match '^(README|LICENSE|COPYING|NOTICE)' }
    foreach ($notice in $assetNotices) {
        $relative = $notice.FullName.Substring((Join-Path $repo 'assets').Length + 1)
        Copy-RequiredFile $notice.FullName (Join-Path $documentation (Join-Path 'data-provenance' $relative))
    }

    $dependencyPackagesById = @{}
    Push-Location $repo
    try {
        foreach ($metadataTarget in @(
            @{ Triple = $Target; Roots = @('qingjian-windows-server', 'qingjian-windows-tsf', 'qingjian-settings') },
            @{ Triple = $Target32; Roots = @('qingjian-windows-tsf') }
        )) {
            $metadataRaw = & cargo metadata --format-version 1 --locked --filter-platform $metadataTarget.Triple
            if ($LASTEXITCODE -ne 0) { throw "cargo metadata ($($metadataTarget.Triple)) 失败，退出码 $LASTEXITCODE" }
            $metadata = ($metadataRaw -join "`n") | ConvertFrom-Json
            foreach ($package in Get-ReachablePackages $metadata $metadataTarget.Roots) {
                $dependencyPackagesById[$package.id] = $package
            }
        }
    } finally { Pop-Location }

    foreach ($package in ($dependencyPackagesById.Values | Sort-Object name, version)) {
        $manifestDirectory = Split-Path -Parent $package.manifest_path
        $licenseFiles = @(Get-ChildItem -LiteralPath $manifestDirectory -File -Recurse -Include 'LICENSE*', 'COPYING*', 'NOTICE*' -ErrorAction SilentlyContinue)
        if ($licenseFiles.Count -gt 0) {
            $packageDirectoryName = "$($package.name)-$($package.version)"
            foreach ($licenseFile in $licenseFiles) {
                $relative = $licenseFile.FullName.Substring($manifestDirectory.Length + 1).Replace('\', '-')
                Copy-RequiredFile $licenseFile.FullName (Join-Path $documentation (Join-Path "dependency-licenses\$packageDirectoryName" $relative))
            }
        }
    }
    $dependencyLines = @(
        '# 第三方依赖与数据许可',
        '',
        '项目源代码许可见 LICENSE 与 NOTICE。下表来自 x64 和 x86 Windows 目标的锁定 Cargo 依赖图；许可证字段取自各包清单。',
        '依赖目录中的 LICENSE、COPYING 与 NOTICE 原文位于 dependency-licenses/；assets 下的 README 与许可原文位于 data-provenance/。',
        '',
        '| crate | 版本 | license 字段 | 收集的原文文件 |',
        '| --- | --- | --- | --- |'
    )
    foreach ($package in ($dependencyPackagesById.Values | Sort-Object name, version)) {
        $license = if ($package.license) { $package.license } elseif ($package.license_file) { "license-file: $($package.license_file)" } else { '未在 Cargo 清单声明' }
        $licenseDirectory = Join-Path $documentation "dependency-licenses\$($package.name)-$($package.version)"
        $licenseCount = if (Test-Path -LiteralPath $licenseDirectory) { (Get-ChildItem -LiteralPath $licenseDirectory -File -Recurse).Count } else { 0 }
        $dependencyLines += "| $($package.name) | $($package.version) | $license | $licenseCount 个原文文件 |"
    }
    $dependencyLines | Set-Content -LiteralPath (Join-Path $documentation 'THIRD-PARTY-NOTICES.md') -Encoding UTF8

    $dataLock = Get-Content -LiteralPath (Join-Path $repo 'tools\release\data.lock') -Encoding UTF8
    $dataTag = ($dataLock | Where-Object { $_ -match '^\s*tag\s*=\s*(\S+)' } | Select-Object -First 1) -replace '^\s*tag\s*=\s*', ''
    $dataHash = ($dataLock | Where-Object { $_ -match '^\s*qingjian-data\.tar\.gz\s*=\s*([0-9a-fA-F]{64})' } | Select-Object -First 1) -replace '^\s*qingjian-data\.tar\.gz\s*=\s*', ''
    Push-Location $repo
    try {
        $gitHead = (& git rev-parse HEAD)
        if ($LASTEXITCODE -ne 0) { throw "git rev-parse 失败，退出码 $LASTEXITCODE" }
        $gitStatus = @(& git status --porcelain --untracked-files=all)
        if ($LASTEXITCODE -ne 0) { throw "git status 失败，退出码 $LASTEXITCODE" }
        $rustVersion = (& rustc --version)
        if ($LASTEXITCODE -ne 0) { throw "rustc --version 失败，退出码 $LASTEXITCODE" }
    } finally { Pop-Location }
    @(
        'CtrlLingo · 译点 Windows 安装包构建信息',
        "Version: 0.1.0",
        "Source commit: $gitHead",
        "Product data: $dataTag ($dataHash)",
        "Target: $Target / $Target32",
        "Rust: $rustVersion",
        "Built UTC: $([DateTime]::UtcNow.ToString('o'))",
        'The separate CtrlLingo-0.1.0-source.zip contains git tracked files and non-ignored untracked files from this working tree.',
        "Working tree dirty: $(if ($gitStatus.Count -gt 0) { 'yes' } else { 'no' })",
        "Dirty paths: $($gitStatus.Count)"
    ) | Set-Content -LiteralPath (Join-Path $documentation 'SOURCE-INFO.txt') -Encoding UTF8

    $stageSums = Join-Path $documentation 'SHA256SUMS.txt'
    Add-HashLines $stage $stageSums

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $sourceZipPath = Join-Path $outputRoot 'CtrlLingo-0.1.0-source.zip'
    if (Test-Path -LiteralPath $sourceZipPath) { Remove-Item -LiteralPath $sourceZipPath -Force }
    $sourceZip = [System.IO.Compression.ZipFile]::Open($sourceZipPath, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        Push-Location $repo
        try {
            $sourceFiles = & git -c core.quotepath=false ls-files --cached --others --exclude-standard
            if ($LASTEXITCODE -ne 0) { throw "git ls-files 失败，退出码 $LASTEXITCODE" }
            foreach ($file in $sourceFiles) {
                $sourcePath = Join-Path $repo $file
                if (Test-Path -LiteralPath $sourcePath -PathType Leaf) {
                    [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($sourceZip, $sourcePath, $file.Replace('\', '/'), [System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
                }
            }
        } finally { Pop-Location }
    } finally { $sourceZip.Dispose() }

    $iscc = $env:CTRLLINGO_ISCC
    if (-not $iscc) { $iscc = (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source }
    if (-not $iscc) { throw '找不到 ISCC.exe；请通过 CTRLLINGO_ISCC 指定 Inno Setup 编译器。' }
    Invoke-CheckedNative $iscc @("/DPackageRoot=$stage", "/DOutputDirectory=$outputRoot", $iss) 'Inno Setup 编译'

    $setup = Join-Path $outputRoot 'CtrlLingo-0.1.0-windows-x64-setup.exe'
    if (-not (Test-Path -LiteralPath $setup -PathType Leaf)) { throw "ISCC 没有生成安装包：$setup" }
    $outputSums = @($setup, $sourceZipPath) | ForEach-Object {
        "{0}  {1}" -f (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant(), [System.IO.Path]::GetFileName($_)
    }
    $outputSums | Set-Content -LiteralPath (Join-Path $outputRoot 'SHA256SUMS.txt') -Encoding UTF8
    Write-Host "已生成安装包与源快照：$outputRoot" -ForegroundColor Green
} finally {
    if ($null -eq $previousUiAccess) { Remove-Item Env:QINGJIAN_UIACCESS -ErrorAction SilentlyContinue }
    else { $env:QINGJIAN_UIACCESS = $previousUiAccess }
}
