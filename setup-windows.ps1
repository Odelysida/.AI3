#!/usr/bin/env pwsh

# TribeChain Windows Setup Script
Write-Host "=== TribeChain Windows Setup ===" -ForegroundColor Green

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

if (-not $isAdmin) {
    Write-Host "Warning: Not running as administrator. Some operations may fail." -ForegroundColor Yellow
}

# Function to check if a command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Install Chocolatey if not present
if (-not (Test-Command choco)) {
    Write-Host "Installing Chocolatey..." -ForegroundColor Yellow
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    refreshenv
}

# Install Rust using rustup
if (-not (Test-Command cargo)) {
    Write-Host "Installing Rust..." -ForegroundColor Yellow
    
    # Download and run rustup installer
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    
    Write-Host "Downloading rustup installer..."
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
    
    Write-Host "Running rustup installer..."
    Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait
    
    # Add Rust to PATH for current session
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
    
    Write-Host "Rust installed successfully!" -ForegroundColor Green
} else {
    Write-Host "Rust is already installed." -ForegroundColor Green
    cargo --version
}

# Install Git if not present
if (-not (Test-Command git)) {
    Write-Host "Installing Git..." -ForegroundColor Yellow
    choco install git -y
    refreshenv
}

# Install Visual Studio Build Tools (required for some Rust crates)
Write-Host "Checking for Visual Studio Build Tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $buildTools = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $buildTools) {
        Write-Host "Installing Visual Studio Build Tools..." -ForegroundColor Yellow
        choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64" -y
    }
} else {
    Write-Host "Installing Visual Studio Build Tools..." -ForegroundColor Yellow
    choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64" -y
}

# Build TribeChain
Write-Host "Building TribeChain..." -ForegroundColor Yellow
if (Test-Path "Cargo.toml") {
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Host "TribeChain built successfully!" -ForegroundColor Green
        Write-Host "Executable location: target\release\tribechain.exe" -ForegroundColor Cyan
    } else {
        Write-Host "Build failed. Please check the error messages above." -ForegroundColor Red
    }
} else {
    Write-Host "Cargo.toml not found. Make sure you're in the TribeChain directory." -ForegroundColor Red
}

# Create run script
$runScript = @"
@echo off
echo Starting TribeChain Node...
target\release\tribechain.exe node --port 8333 --data-dir ./data
pause
"@

$runScript | Out-File -FilePath "run-tribechain.bat" -Encoding ASCII

Write-Host ""
Write-Host "=== Setup Complete ===" -ForegroundColor Green
Write-Host "To start TribeChain:" -ForegroundColor Cyan
Write-Host "  1. Run: .\run-tribechain.bat" -ForegroundColor White
Write-Host "  2. Or: target\release\tribechain.exe --help" -ForegroundColor White
Write-Host ""
Write-Host "For ESP32 development, see esp32/ directory" -ForegroundColor Yellow 