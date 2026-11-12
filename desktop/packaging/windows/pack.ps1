# Packs the Windows release from a built binary: a zip with iibrain.exe,
# the licences and the icon.
#
#   packaging/windows/pack.ps1 -Version 0.1.0 -Binary target\release\iibrain.exe -Dist dist
param(
  [Parameter(Mandatory = $true)][string]$Version,
  [Parameter(Mandatory = $true)][string]$Binary,
  [Parameter(Mandatory = $true)][string]$Dist
)
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Resolve-Path (Join-Path $here "..\..\..")
New-Item -ItemType Directory -Force $Dist | Out-Null
$stage = Join-Path ([System.IO.Path]::GetTempPath()) ("iibrain-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item $Binary (Join-Path $stage "iibrain.exe")
Copy-Item (Join-Path $root "LICENSE-APACHE") $stage
Copy-Item (Join-Path $root "LICENSE-MIT") $stage
Copy-Item (Join-Path $root "THIRD-PARTY-NOTICES.md") $stage
Copy-Item (Join-Path $here "..\icon\icon_256.png") (Join-Path $stage "iibrain.png")
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath (Join-Path $Dist "iibrain-$Version-windows-x86_64.zip") -Force
Remove-Item -Recurse -Force $stage
Get-ChildItem $Dist
