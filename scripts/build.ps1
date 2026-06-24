# Build script for VoxelNaut on Windows
# Requires: Rust (rustup), Git

param(
    [switch]$Release,
    [switch]$Clean,
    [switch]$Watch,
    [string]$Features = ""
)

$ErrorActionPreference = "Stop"

Write-Host "=== VoxelNaut Build Script ===" -ForegroundColor Cyan
Write-Host ""

# Check Rust installation
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Rust is not installed!" -ForegroundColor Red
    Write-Host "Install from: https://rustup.rs" -ForegroundColor Yellow
    exit 1
}

$env:CARGO_HTTP_MULTIPLEXING = "false"

# Build type
if ($Release) {
    $BUILD_MODE = "release"
    $FEATURES = "launcher/game"
    Write-Host "Building in RELEASE mode" -ForegroundColor Green
} else {
    $BUILD_MODE = "debug"
    $FEATURES = "launcher/game"
    Write-Host "Building in DEBUG mode" -ForegroundColor Yellow
}

# Clean if requested
if ($Clean) {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    cargo clean
}

# Determine features
if ($Features -ne "") {
    $BUILD_FEATURES = $Features
} else {
    $BUILD_FEATURES = $FEATURES
}

Write-Host ""
Write-Host "Features: $BUILD_FEATURES" -ForegroundColor Gray
Write-Host ""

# Build
Write-Host "Compiling VoxelNaut..." -ForegroundColor Cyan

if ($BUILD_MODE -eq "release") {
    cargo build --release --features $BUILD_FEATURES 2>&1
} else {
    cargo build --features $BUILD_FEATURES 2>&1
}

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "Build FAILED!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Build SUCCESSFUL!" -ForegroundColor Green

# Find executable
$EXE_NAME = if ($BUILD_MODE -eq "release") { "voxelnaut.exe" } else { "voxelnaut.exe" }
$TARGET_DIR = if ($BUILD_MODE -eq "release") { "target\release" } else { "target\debug" }
$EXE_PATH = Join-Path $TARGET_DIR $EXE_NAME

if (Test-Path $EXE_PATH) {
    $SIZE = (Get-Item $EXE_PATH).Length / 1MB
    Write-Host ""
    Write-Host "Output: $EXE_PATH" -ForegroundColor Cyan
    Write-Host "Size: $([math]::Round($SIZE, 2)) MB" -ForegroundColor Cyan
    
    Write-Host ""
    Write-Host "To run:" -ForegroundColor Yellow
    Write-Host "  .\$EXE_PATH" -ForegroundColor White
} else {
    Write-Host "Warning: Executable not found at expected path" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Build Complete ===" -ForegroundColor Cyan