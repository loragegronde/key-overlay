<#
.SYNOPSIS
    Runs a Tauri command inside the MSVC build environment.

.DESCRIPTION
    rustc only receives the Windows SDK library paths (ucrt.lib, um) when the
    MSVC developer environment is loaded. Without it, linking fails with
    "LNK1104: cannot open file 'ucrt.lib'". This script imports the variables
    that vcvars64.bat sets, then hands off to the requested npm script.

.EXAMPLE
    .\dev.ps1              # tauri dev
    .\dev.ps1 build        # tauri build (installers)
    .\dev.ps1 portable     # cargo build --release, portable .exe only
#>
param(
    [ValidateSet("dev", "build", "portable")]
    [string]$Command = "dev"
)

$ErrorActionPreference = "Stop"

function Find-VcVars {
    $installRoots = @()

    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $installRoots += & $vswhere -prerelease -products * -format value -property installationPath 2>$null
    }

    # vswhere does not report every Build Tools install, so also scan the default roots.
    foreach ($root in @("${env:ProgramFiles(x86)}\Microsoft Visual Studio", "$env:ProgramFiles\Microsoft Visual Studio")) {
        if (-not (Test-Path $root)) { continue }
        $installRoots += Get-ChildItem $root -Directory -ErrorAction SilentlyContinue |
            ForEach-Object { Get-ChildItem $_.FullName -Directory -ErrorAction SilentlyContinue } |
            Select-Object -ExpandProperty FullName
    }

    # Edition directory names ("2019", "18") do not sort meaningfully, so rank by
    # the highest MSVC toolset each install provides.
    $installRoots |
        Where-Object { $_ } |
        Select-Object -Unique |
        ForEach-Object {
            $vcvars = Join-Path $_ "VC\Auxiliary\Build\vcvars64.bat"
            if (-not (Test-Path $vcvars)) { return }
            $toolset = Get-ChildItem (Join-Path $_ "VC\Tools\MSVC") -Directory -ErrorAction SilentlyContinue |
                ForEach-Object { $_.Name -as [version] } |
                Where-Object { $_ } |
                Sort-Object -Descending |
                Select-Object -First 1
            if ($toolset) { [pscustomobject]@{ Path = $vcvars; Toolset = $toolset } }
        } |
        Sort-Object Toolset -Descending |
        Select-Object -First 1 -ExpandProperty Path
}

$vcvars = Find-VcVars
if (-not $vcvars) {
    throw "vcvars64.bat not found. Install Visual Studio Build Tools with the 'Desktop development with C++' workload."
}

function Compress-Path {
    # vcvars appends the same entries on every run. Past ~8191 chars cmd truncates
    # PATH, which breaks both the vcvars call and npm's node_modules\.bin lookup.
    $env:PATH = (($env:PATH -split ';' | Where-Object { $_ } | Select-Object -Unique) -join ';')
}

Write-Host "Loading MSVC environment from:`n  $vcvars" -ForegroundColor Cyan

Compress-Path

cmd /c "call `"$vcvars`" >nul && set" | ForEach-Object {
    if ($_ -match '^([^=]+)=(.*)$') {
        Set-Item -Path "env:$($matches[1])" -Value $matches[2]
    }
}

$cargoBin = "$env:USERPROFILE\.cargo\bin"
if (Test-Path $cargoBin) { $env:PATH = "$cargoBin;$env:PATH" }

Compress-Path

if (-not $env:LIB) {
    throw "LIB is still empty; the MSVC environment did not load correctly."
}

Set-Location $PSScriptRoot

# Tauri, Vite and cargo all log progress to stderr; under "Stop" that would
# abort the run.
$ErrorActionPreference = "Continue"

if ($Command -eq "portable") {
    # The bundler is skipped entirely: this produces only the single portable
    # executable, which is the primary Windows artifact.
    Write-Host "Running: npm run build; cargo build --release" -ForegroundColor Cyan
    npm run build
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Push-Location (Join-Path $PSScriptRoot "src-tauri")
    cargo build --release
    $code = $LASTEXITCODE
    Pop-Location
    if ($code -ne 0) { exit $code }

    $exe = Join-Path $PSScriptRoot "src-tauri\target\release\key-overlay.exe"
    if (Test-Path $exe) {
        Write-Host ("`n{0}`n{1:N2} MiB" -f $exe, ((Get-Item $exe).Length / 1MB)) -ForegroundColor Green
    }
    exit 0
}

Write-Host "Running: npm run tauri $Command" -ForegroundColor Cyan
npm run tauri $Command
