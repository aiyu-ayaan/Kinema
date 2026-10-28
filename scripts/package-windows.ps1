# The server for a Windows PC, in one folder: the backend, the web UI and ffmpeg.
#   scripts/package-windows.ps1 [-Version 0.1.0]   -> dist\kinema-server-<version>-windows-x64.zip
# Same layout as the Linux tarball (scripts/package-server.sh): kinema.exe finds web\, ffmpeg.exe and
# ffprobe.exe next to itself. Needs Rust (MSVC toolchain), Node and pnpm.
Param(
    [string]$Version = (node -p "require('./package.json').version")
)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

function Lock($key) { (Get-Content scripts/ffmpeg.lock | Where-Object { $_ -like "$key=*" }) -replace "^$key=", '' }
$ffVersion = Lock 'version'
$source = Lock 'source'
$name = "kinema-server-$Version-windows-x64"
$work = Join-Path ([IO.Path]::GetTempPath()) "kinema-pack-$([guid]::NewGuid())"
$out = Join-Path $work $name
New-Item -ItemType Directory -Path $out | Out-Null

try {
    pnpm install --frozen-lockfile --filter '@kinema/web'
    pnpm --filter '@kinema/web' build
    cargo build --release -p backend
    Copy-Item target/release/kinema.exe $out
    Copy-Item apps/web/dist (Join-Path $out 'web') -Recurse

    $archive = "ffmpeg-n$ffVersion-latest-win64-gpl-$ffVersion.zip"
    Invoke-WebRequest "$source/$archive" -OutFile (Join-Path $work $archive)
    Invoke-WebRequest "$source/checksums.sha256" -OutFile (Join-Path $work 'checksums.sha256')
    $line = Get-Content (Join-Path $work 'checksums.sha256') | Where-Object { $_ -match " \*?$([regex]::Escape($archive))$" }
    if (-not $line) { throw "no checksum published for $archive" }
    $expected = ($line -split '\s+')[0]
    $actual = (Get-FileHash (Join-Path $work $archive) -Algorithm SHA256).Hash
    if ($actual -ne $expected) { throw "checksum mismatch for $archive" }

    Expand-Archive (Join-Path $work $archive) -DestinationPath $work
    $ff = Join-Path $work ($archive -replace '\.zip$', '')
    Copy-Item (Join-Path $ff 'bin/ffmpeg.exe'), (Join-Path $ff 'bin/ffprobe.exe') $out
    $licenses = Join-Path $out 'licenses'
    New-Item -ItemType Directory -Path $licenses | Out-Null
    Copy-Item LICENSE (Join-Path $licenses 'kinema.txt')
    Copy-Item (Join-Path $ff 'LICENSE.txt') (Join-Path $licenses 'ffmpeg.txt')
    "ffmpeg $ffVersion, $archive from $source" | Set-Content (Join-Path $licenses 'ffmpeg-source.txt')

    New-Item -ItemType Directory -Path dist -Force | Out-Null
    $zip = "dist/$name.zip"
    Remove-Item $zip -ErrorAction SilentlyContinue
    Compress-Archive -Path $out -DestinationPath $zip
    Write-Host $zip
} finally {
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
}
