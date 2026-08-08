# Load MSVC environment (required on Windows for native crates) and build/run.
param(
    [switch]$Release,
    [switch]$Run,
    # Keep the process attached to this console (closes when the shell closes).
    [switch]$Foreground
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

$buildArgs = @("build")
if ($Release) { $buildArgs += "--release" }

Write-Host "cargo $($buildArgs -join ' ')"
cargo @buildArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if ($Run) {
    $exe = if ($Release) {
        Join-Path $PSScriptRoot "target\release\key-overlay.exe"
    } else {
        Join-Path $PSScriptRoot "target\debug\key-overlay.exe"
    }
    if (-not (Test-Path $exe)) {
        Write-Error "Built executable not found: $exe"
        exit 1
    }

    if ($Foreground) {
        Write-Host "Running in foreground: $exe"
        & $exe
        exit $LASTEXITCODE
    }

    # Detached GUI process — closing this PowerShell window will not kill the app.
    Write-Host "Starting Key Overlay (detached): $exe"
    Start-Process -FilePath $exe -WorkingDirectory $PSScriptRoot
    Write-Host "App is running independently. You can close this terminal."
}
