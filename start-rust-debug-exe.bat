@echo off
REM -----------------------------------------------------------------------
REM start-rust-debug-exe.bat  — Launch the built debug EXE with correct CWD
REM -----------------------------------------------------------------------

cd /d "%~dp0"

echo.
echo [Rust App] Starting debug EXE from: %CD%

if not exist "target\debug\embroidery-catalogue.exe" (
    echo [Rust App] Debug EXE not found. Building it now...
    cargo tauri build --debug --no-bundle
    if errorlevel 1 (
        echo ERROR: Failed to build debug EXE.
        pause
        exit /b 1
    )
)

echo [Rust App] Launching target\debug\embroidery-catalogue.exe
"target\debug\embroidery-catalogue.exe"
set "APP_EXIT=%ERRORLEVEL%"

echo.
echo [Rust App] EXE exited with code: %APP_EXIT%
if not "%RUST_APP_NO_PAUSE%"=="1" pause
exit /b %APP_EXIT%
