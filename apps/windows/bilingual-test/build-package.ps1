param(
    [string]$Target = 'x86_64-pc-windows-msvc',
    [string]$Target32 = 'i686-pc-windows-msvc',
    [string]$BinaryDirectory,
    [string]$OutputDirectory = 'target\bilingual-test',
    [string]$PreviewDirectory = 'target\bilingual-preview',
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$env:QINGJIAN_UIACCESS = '0'
Push-Location $repo
try {
    if (-not $SkipBuild) {
        cargo build --release --locked --target $Target -p qingjian-windows-server -p qingjian-windows-tsf -p qingjian-settings
        if ($LASTEXITCODE -ne 0) { throw '64-bit build failed.' }
        cargo build --release --locked --target $Target32 -p qingjian-windows-tsf
        if ($LASTEXITCODE -ne 0) { throw '32-bit build failed.' }
    }
    $release = if ($BinaryDirectory) { (Resolve-Path $BinaryDirectory).Path } else { Join-Path $repo "target\$Target\release" }
    $release32 = Join-Path $repo "target\$Target32\release"
    $required = @('qingjian-server.exe', 'qingjian_tsf.dll', 'qingjian-settings.exe')
    foreach ($name in $required) {
        if (-not (Test-Path -LiteralPath (Join-Path $release $name))) { throw "Missing binary: $name" }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $release32 'qingjian_tsf.dll'))) { throw 'Missing 32-bit TSF DLL.' }
    if (-not (Test-Path -LiteralPath 'data\generated\dict.qj')) { throw 'Fetch and verify the data.lock product bundle first.' }

    $outputRoot = if ([System.IO.Path]::IsPathRooted($OutputDirectory)) { $OutputDirectory } else { Join-Path $repo $OutputDirectory }
    $package = Join-Path $outputRoot 'QingjianBilingualTest'
    if (Test-Path -LiteralPath $package) { throw "Output already exists: $package" }
    $app = Join-Path $package 'app'
    New-Item -ItemType Directory -Path $app -Force | Out-Null
    foreach ($name in $required) { Copy-Item -LiteralPath (Join-Path $release $name) -Destination $app }
    Copy-Item -LiteralPath (Join-Path $release32 'qingjian_tsf.dll') -Destination (Join-Path $app 'qingjian_tsf32.dll')
    foreach ($name in @('data', 'assets')) {
        Copy-Item -LiteralPath (Join-Path $repo $name) -Destination $app -Recurse
    }
    foreach ($name in @('install-test.ps1', 'uninstall-test.ps1', 'install-test.cmd', 'uninstall-test.cmd', 'default-config.toml', '先看这里.md')) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $package
    }
    Copy-Item -LiteralPath 'docs\notes\双语测试验收.md', 'docs\notes\双语版本验证记录.md', 'docs\notes\Ctrl按需翻译与费用验证记录.md', 'LICENSE' -Destination $package
    if ($PreviewDirectory -and (Test-Path -LiteralPath $PreviewDirectory)) {
        Copy-Item -LiteralPath $PreviewDirectory -Destination (Join-Path $package '预览') -Recurse
    }
    $info = @(
        'Personal Windows bilingual test fork; not an official release.',
        'Upstream base: c97ff2b61933d6fcd845ee4f4f470faf3ea83bf6',
        "Fork source commit: $(git rev-parse HEAD)",
        'Rollback tag: backup/bilingual-live-before-ondemand-20261019 (2c0ed70221b90fb7f2866fe96564ed4b5258acae)',
        "Targets: $Target / $Target32",
        "Rust: $(rustc --version)",
        "Rust flags: $env:RUSTFLAGS",
        "Build helper optimization: $env:CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_OPT_LEVEL",
        "Built UTC: $([DateTime]::UtcNow.ToString('o'))",
        'Product data: data-v7; SHA256 e259f8bc2a5456c92484c5d4362ca56b3e46b3d2e1a0d463788091a039d79e40',
        'Source: source.zip includes the modified working-tree sources; dependencies are pinned in Cargo.lock.'
    )
    $info | Set-Content -LiteralPath (Join-Path $package 'SOURCE-INFO.txt') -Encoding UTF8

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $sourceZip = [System.IO.Compression.ZipFile]::Open((Join-Path $package 'source.zip'), 'Create')
    try {
        $tracked = git -c core.quotepath=false ls-files -co --exclude-standard
        foreach ($file in $tracked) {
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($sourceZip, (Join-Path $repo $file), $file, 'Optimal') | Out-Null
        }
    } finally { $sourceZip.Dispose() }

    $manifest = Get-ChildItem -LiteralPath $package -File -Recurse | ForEach-Object {
        $relative = $_.FullName.Substring($package.Length + 1).Replace('\', '/')
        "{0}  {1}" -f (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $relative
    }
    $manifest | Set-Content -LiteralPath (Join-Path $package 'SHA256SUMS.txt') -Encoding UTF8
    $zipPath = "$package-windows-x64.zip"
    [System.IO.Compression.ZipFile]::CreateFromDirectory($package, $zipPath, 'Optimal', $true)
    Get-FileHash -LiteralPath $zipPath -Algorithm SHA256
} finally { Pop-Location }
