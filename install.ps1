# Install goodissues CLI on Windows.
# Usage: irm https://raw.githubusercontent.com/goodway/goodissues/main/cli-zig/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Version = if ($env:GOODISSUES_VERSION) { $env:GOODISSUES_VERSION } else { "latest" }
$InstallDir = if ($env:GOODISSUES_INSTALL_DIR) { $env:GOODISSUES_INSTALL_DIR } else { "$env:LOCALAPPDATA\goodissues" }
$BaseUrl = if ($env:GOODISSUES_BASE_URL) { $env:GOODISSUES_BASE_URL } else { "https://github.com/agoodway/goodissues_cli/releases/download" }

$Arch = if ([Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "amd64" }
} else {
    Write-Error "32-bit Windows is not supported"; exit 1
}

$Binary = "goodissues-windows-${Arch}.exe"

if ($Version -eq "latest") {
    $Url = "${BaseUrl}/latest/download/${Binary}"
} else {
    $Url = "${BaseUrl}/v${Version}/${Binary}"
}

Write-Host "Downloading goodissues for windows/${Arch}..."

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$OutFile = Join-Path $InstallDir "goodissues.exe"
Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    $env:PATH = "$env:PATH;$InstallDir"
    Write-Host "Added $InstallDir to PATH"
}

Write-Host "goodissues installed to $OutFile"
& $OutFile --version
