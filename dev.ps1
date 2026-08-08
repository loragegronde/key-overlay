# Load MSVC environment (required on Windows for native crates) and build/run.
param(
    [switch]$Release,
    [switch]$Run
)

$ErrorActionPreference = "Stop"

function Import-VcVars {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) { return $false }
    $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $vs) { return $false }
    $vcvars = Join-Path $vs "VC\Auxiliary\Build\vcvars64.bat"
    if (-not (Test-Path $vcvars)) { return $false }
    cmd /c "`"$vcvars`" && set" | ForEach-Object {
        if ($_ -match "^(.*?)=(.*)$") {
            [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2])
        }
    }
    return $true
}

if ($IsWindows -or $env:OS -match "Windows") {
    [void](Import-VcVars)
}

$args = @("build")
if ($Release) { $args += "--release" }

Write-Host "cargo $($args -join ' ')"
cargo @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if ($Run) {
    if ($Release) {
        cargo run --release
    } else {
        cargo run
    }
}
