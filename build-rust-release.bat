@echo off
REM -----------------------------------------------------------------------
REM build-rust-release.bat  —  Build the release installer for distribution
REM
REM Produces:
REM   target/release/bundle/msi/*.msi                   — Windows Installer
REM   target/release/bundle/nsis/*-setup.exe            — NSIS Installer
REM -----------------------------------------------------------------------

cd /d "%~dp0"

echo.
echo [Rust Build] Building release target from: %CD%

where cargo >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust cargo was not found in PATH.
    echo Install Rust from https://rustup.rs and retry.
    pause
    exit /b 1
)

cargo tauri build --verbose
set "BUILD_EXIT=%ERRORLEVEL%"

if %BUILD_EXIT% equ 0 (
    echo.
    echo [Rust Build] Release build successful!
    echo.
    echo Output files:
    echo   MSI installer:    target\release\bundle\msi\
    echo   NSIS installer:   target\release\bundle\nsis\
) else (
    echo.
    echo [Rust Build] Release build FAILED with exit code %BUILD_EXIT%.
)

if not "%RUST_APP_NO_PAUSE%"=="1" pause
exit /b %BUILD_EXIT%