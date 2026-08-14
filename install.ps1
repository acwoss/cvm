# install.ps1 - install cvm (Claude Virtualenv Manager) on Windows
#
# Usage:
#   powershell -c "irm https://getcvm.com/install.ps1 | iex"
#
# Environment overrides:
#   CVM_INSTALL_DIR   Install prefix (default: $HOME\.cvm)
#   CVM_VERSION       Release tag to install (default: latest)

$ErrorActionPreference = "Stop"

$Repo = "acwoss/cvm"
$BinName = "cvm.exe"
$InstallDir = if ($env:CVM_INSTALL_DIR) { $env:CVM_INSTALL_DIR } else { Join-Path $HOME ".cvm" }
$BinDir = Join-Path $InstallDir "bin"
$Version = if ($env:CVM_VERSION) { $env:CVM_VERSION } else { "latest" }

function Info($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Warn($msg) { Write-Host "warning: $msg" -ForegroundColor Yellow }
function Die($msg) { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }

function Get-Target {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
    if ($arch -ne [System.Runtime.InteropServices.Architecture]::X64) {
        Die "unsupported CPU architecture: $arch. Install manually via 'cargo install cvm' or download a release from https://github.com/$Repo/releases"
    }
    return "x86_64-pc-windows-msvc"
}

function Add-ProfileHook {
    if (-not (Test-Path $PROFILE)) {
        New-Item -ItemType File -Path $PROFILE -Force | Out-Null
    }

    $hook = 'cvm init powershell | Out-String | Invoke-Expression'
    $content = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue

    if ($content -notmatch [regex]::Escape($BinDir)) {
        Add-Content -Path $PROFILE -Value ""
        Add-Content -Path $PROFILE -Value "# Added by cvm installer"
        Add-Content -Path $PROFILE -Value "`$env:PATH = `"$BinDir;`$env:PATH`""
        Info "Added $BinDir to PATH in $PROFILE"
    }

    if ($content -notmatch "cvm init") {
        Add-Content -Path $PROFILE -Value $hook
        Info "Added cvm shell hook to $PROFILE"
    }
}

function Main {
    $target = Get-Target
    $archive = "cvm-$target.zip"

    if ($Version -eq "latest") {
        $url = "https://github.com/$Repo/releases/latest/download/$archive"
    } else {
        $url = "https://github.com/$Repo/releases/download/$Version/$archive"
    }

    Info "Target: $target"
    Info "Downloading $url"

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tmpDir | Out-Null
    $archivePath = Join-Path $tmpDir $archive

    try {
        try {
            Invoke-WebRequest -Uri $url -OutFile $archivePath -UseBasicParsing
        } catch {
            Die "download failed. Check that a release exists for '$target' at https://github.com/$Repo/releases"
        }

        Expand-Archive -Path $archivePath -DestinationPath $tmpDir -Force

        $extractedBin = Get-ChildItem -Path $tmpDir -Filter $BinName -Recurse | Select-Object -First 1
        if (-not $extractedBin) {
            Die "could not find the '$BinName' binary inside the downloaded archive"
        }

        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
        Copy-Item -Path $extractedBin.FullName -Destination (Join-Path $BinDir $BinName) -Force
        Info "Installed $BinName to $BinDir\$BinName"

        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($userPath -notlike "*$BinDir*") {
            [Environment]::SetEnvironmentVariable("PATH", "$BinDir;$userPath", "User")
            Info "Added $BinDir to your User PATH"
        }
        $env:PATH = "$BinDir;$env:PATH"

        Add-ProfileHook
        Info "Done. Restart PowerShell, or run: . `$PROFILE"
    } finally {
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Main
