param([string]$Source = (Join-Path $PSScriptRoot '..\assets\brand\ctrllingo-mark.png'))
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$iconPath = Join-Path $PSScriptRoot '..\apps\windows\tsf\resources\ctrllingo.ico'
$image = [System.Drawing.Image]::FromFile((Resolve-Path $Source))
$frames = @()
try {
    foreach ($size in @(16, 20, 24, 32, 48, 64, 128, 256)) {
        $bitmap = New-Object System.Drawing.Bitmap($size, $size)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $stream = New-Object System.IO.MemoryStream
        try {
            $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $graphics.DrawImage($image, 0, 0, $size, $size)
            $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
            $frames += ,@($size, $stream.ToArray())
        } finally { $stream.Dispose(); $graphics.Dispose(); $bitmap.Dispose() }
    }
    $output = [System.IO.File]::Create($iconPath)
    $writer = New-Object System.IO.BinaryWriter($output)
    try {
        $writer.Write([uint16]0); $writer.Write([uint16]1); $writer.Write([uint16]$frames.Count)
        $offset = 6 + 16 * $frames.Count
        foreach ($frame in $frames) {
            $edge = if ($frame[0] -eq 256) { 0 } else { $frame[0] }
            $writer.Write([byte]$edge); $writer.Write([byte]$edge)
            $writer.Write([byte]0); $writer.Write([byte]0)
            $writer.Write([uint16]1); $writer.Write([uint16]32)
            $writer.Write([uint32]$frame[1].Length); $writer.Write([uint32]$offset)
            $offset += $frame[1].Length
        }
        foreach ($frame in $frames) { $writer.Write([byte[]]$frame[1]) }
    } finally { $writer.Dispose(); $output.Dispose() }
} finally { $image.Dispose() }
Write-Output $iconPath
