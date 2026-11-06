# Packages a Windows build as valler-<version>-windows-<arch>.zip.
#
# usage: ci/package-windows.ps1 <path\to\valler.exe> <arch> <version> <out-dir>
param(
    [Parameter(Mandatory = $true)][string]$Bin,
    [Parameter(Mandatory = $true)][string]$Arch,
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Out
)
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$name = "valler-$Version-windows-$Arch"
$stage = Join-Path ([System.IO.Path]::GetTempPath()) $name
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force -Path $stage, $Out | Out-Null

Copy-Item $Bin (Join-Path $stage "Valler.exe")
Copy-Item (Join-Path $root "README.md") $stage

$zip = Join-Path $Out "$name.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zip
Write-Output $zip
