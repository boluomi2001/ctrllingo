[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$lockPath = Join-Path $PSScriptRoot 'release\data.lock'
if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) { throw "缺少产品数据锁文件：$lockPath" }

$lock = @{}
foreach ($line in Get-Content -LiteralPath $lockPath -Encoding UTF8) {
    if ($line -match '^\s*(tag|qingjian-data\.tar\.gz)\s*=\s*([^\s#]+)') { $lock[$Matches[1]] = $Matches[2] }
}
if (-not $lock.ContainsKey('tag') -or $lock['tag'] -notmatch '^data-v[0-9]+$') { throw 'data.lock 中缺少有效的数据版本 tag。' }
if (-not $lock.ContainsKey('qingjian-data.tar.gz') -or $lock['qingjian-data.tar.gz'] -notmatch '^[0-9a-fA-F]{64}$') { throw 'data.lock 中缺少有效的 qingjian-data.tar.gz SHA-256。' }

$output = Join-Path $repo 'target\release-data'
$archive = Join-Path $output 'qingjian-data.tar.gz'
$unpacked = Join-Path $output 'unpacked'
$targetRoot = [System.IO.Path]::GetFullPath((Join-Path $repo 'target')).TrimEnd('\')
$output = [System.IO.Path]::GetFullPath($output)
$unpacked = [System.IO.Path]::GetFullPath($unpacked)
if (-not $output.StartsWith($targetRoot + '\', [System.StringComparison]::OrdinalIgnoreCase)) { throw "输出目录必须位于仓库 target 内：$output" }
if (-not $unpacked.StartsWith($targetRoot + '\', [System.StringComparison]::OrdinalIgnoreCase)) { throw "解包目录必须位于仓库 target 内：$unpacked" }
New-Item -ItemType Directory -Path $output -Force | Out-Null
$expected = $lock['qingjian-data.tar.gz'].ToLowerInvariant()
$validCachedArchive = (Test-Path -LiteralPath $archive -PathType Leaf) -and ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -eq $expected)
if (-not $validCachedArchive) {
    if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force }
    $tag = [Uri]::EscapeDataString($lock['tag'])
    $uri = "https://github.com/qingjian-team/qingjian/releases/download/$tag/qingjian-data.tar.gz"
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $uri -OutFile $archive -UseBasicParsing
}

$actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "产品数据 SHA-256 不匹配：期望 $expected，实际 $actual" }

if (Test-Path -LiteralPath $unpacked) { Remove-Item -LiteralPath $unpacked -Recurse -Force }
New-Item -ItemType Directory -Path $unpacked -Force | Out-Null
$tar = Get-Command tar.exe -ErrorAction SilentlyContinue
if (-not $tar) { throw '系统找不到 tar.exe；请使用包含 Windows tar.exe 的受支持 Windows 环境。' }
& $tar.Source '-xzf' $archive '-C' $unpacked
if ($LASTEXITCODE -ne 0) { throw "解压产品数据失败，退出码 $LASTEXITCODE" }

$dataRoot = Join-Path $unpacked 'data'
foreach ($relative in @(
    'generated\dict.qj', 'generated\lm.qj', 'generated\glossary-en.qj',
    'models\hanzhang-zhiwei\hanzhang-zhiwei-small.qjm', 'models\hanzhang-tongbian\hanzhang-tongbian-small.qjm'
)) {
    if (-not (Test-Path -LiteralPath (Join-Path $dataRoot $relative) -PathType Leaf)) { throw "锁定数据包缺少 data\$relative" }
}
if (-not (Get-ChildItem -LiteralPath (Join-Path $dataRoot 'generated\dicts') -Filter '*.qj' -File -ErrorAction SilentlyContinue)) { throw '锁定数据包缺少 data/generated/dicts/*.qj' }
if (-not (Get-ChildItem -LiteralPath (Join-Path $dataRoot 'generated\codes') -Filter '*.qj' -File -ErrorAction SilentlyContinue)) { throw '锁定数据包缺少 data/generated/codes/*.qj' }

$developmentData = Join-Path $repo 'data'
foreach ($subdirectory in @('generated', 'models')) {
    $source = Join-Path $dataRoot $subdirectory
    $destination = Join-Path $developmentData $subdirectory
    if (-not (Test-Path -LiteralPath $source -PathType Container)) { throw "锁定数据包缺少 data\$subdirectory" }
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    Copy-Item -Path (Join-Path $source '*') -Destination $destination -Recurse -Force
}

Write-Host "已下载、校验并解开 $($lock['tag']) 到 $unpacked，并同步到仓库 data\generated 与 data\models" -ForegroundColor Green
