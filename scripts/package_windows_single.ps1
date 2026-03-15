param()

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$primary = Join-Path $root 'dist\windows-native\VaultCanvas.exe'
$fallback = Join-Path $root 'target\release\vaultcanvas_windows_native.exe'
$source = $primary
$target = Join-Path $root 'dist\VaultCanvas-Windows.exe'

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

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
Copy-Item -Path $source -Destination $target -Force

Write-Host "Single EXE ready: $target"
