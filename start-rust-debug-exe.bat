@echo off
REM -----------------------------------------------------------------------
REM start-rust-debug-exe.bat  —  Build and launch the debug EXE with correct CWD
REM -----------------------------------------------------------------------

cd /d "%~dp0"

echo.
echo [Rust App] Starting debug build & launch from: %CD%

REM Ensure cargo and cargo-tauri are reachable in PATH
where cargo >nul 2>&1
if errorlevel 1 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    )
)

echo [Rust App] Building standalone debug EXE (cargo tauri build --debug --no-bundle)...
cargo tauri build --debug --no-bundle
if errorlevel 1 (
    echo ERROR: Failed to build debug EXE.
    if not "%RUST_APP_NO_PAUSE%"=="1" pause
    exit /b 1
)

REM -----------------------------------------------------------------------
REM The debug build runs in Dev mode: it resolves data to <repo>/dev_data/
REM and auto-seeds the database there from the bundled resource on first run.
REM No manual database copy is needed.
REM -----------------------------------------------------------------------

echo [Rust App] Launching target\debug\embroidery-catalogue.exe
"target\debug\embroidery-catalogue.exe"
set "APP_EXIT=%ERRORLEVEL%"

echo.
echo [Rust App] EXE exited with code: %APP_EXIT%
if not "%RUST_APP_NO_PAUSE%"=="1" pause
exit /b %APP_EXIT%