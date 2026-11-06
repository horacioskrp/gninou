# Build complet de gninou : serveur Kotlin (Gradle) -> jar embarque -> client (Cargo).
#
# Usage :
#   .\scripts\build.ps1            # build debug
#   .\scripts\build.ps1 -Release   # build optimise
param([switch]$Release)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

Write-Host "==> 1/2 Compilation du serveur (Gradle)"
Push-Location "$Root\server"
try {
    if (Test-Path "$Root\server\gradlew.bat") { & "$Root\server\gradlew.bat" jar }
    else { gradle jar }
} finally { Pop-Location }

Write-Host "==> 2/2 Compilation du client (Cargo, serveur embarque)"
$env:GNINOU_BUILD_SERVER = "1"
$cargoArgs = @("build", "--manifest-path", "$Root\client\Cargo.toml")
if ($Release) { $cargoArgs += "--release" }
& cargo @cargoArgs

Write-Host "==> Termine."
