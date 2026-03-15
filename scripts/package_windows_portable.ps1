param()

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$primary = Join-Path $root 'dist\windows-native\VaultCanvas.exe'
$fallback = Join-Path $root 'target\release\vaultcanvas_windows_native.exe'
$source = $primary
$target = Join-Path $root 'dist\VaultCanvas-Windows-Portable'

if (!(Test-Path $primary) -and (Test-Path $fallback)) {
  $source = $fallback
}
if ((Test-Path $primary) -and (Test-Path $fallback)) {
  if ((Get-Item $fallback).LastWriteTimeUtc -gt (Get-Item $primary).LastWriteTimeUtc) {
    $source = $fallback
  }
}

if (!(Test-Path $source)) {
  throw "Release EXE not found: $source"
}

if (Test-Path $target) {
  Remove-Item -Recurse -Force $target
}

New-Item -ItemType Directory -Path $target | Out-Null
Copy-Item -Path $source -Destination (Join-Path $target 'VaultCanvas.exe') -Force

$readme = @"
VaultCanvas Windows portable package

1. Double-click VaultCanvas.exe to run.
2. This package ships as a single EXE without extra DLL files.
"@
[System.IO.File]::WriteAllText((Join-Path $target 'README.txt'), $readme, [System.Text.UTF8Encoding]::new($false))

Write-Host "Portable bundle ready: $target"
