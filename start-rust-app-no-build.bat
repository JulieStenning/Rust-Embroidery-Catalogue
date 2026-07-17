@echo off
REM -----------------------------------------------------------------------
REM start-rust-app-no-build.bat  —  Launch prebuilt Rust/Tauri debug app
REM
REM This script does NOT build. It runs:
REM   target\debug\embroidery-catalogue.exe
REM and can auto-start the frontend dev server if needed.
REM
REM Optional overrides before calling this script:
REM   set RUST_APP_NO_PAUSE=1
REM   set RUST_APP_SKIP_DEV_SERVER=1
REM -----------------------------------------------------------------------

cd /d "%~dp0"

echo.
echo [Rust App] Starting prebuilt app from: %CD%
echo [Rust App] Mode: NO BUILD

if not exist "target\debug\embroidery-catalogue.exe" (
    echo ERROR: target\debug\embroidery-catalogue.exe was not found.
    echo Build once first, for example:
    echo   cargo tauri build --debug --no-bundle
    goto :fail
)

if /I not "%RUST_APP_SKIP_DEV_SERVER%"=="1" (
    call :ensure_dev_server
    if errorlevel 1 goto :fail
)

echo [Rust App] Launching target\debug\embroidery-catalogue.exe
"target\debug\embroidery-catalogue.exe"
set "APP_EXIT=%ERRORLEVEL%"

echo.
echo [Rust App] App exited with code: %APP_EXIT%
if /I "%RUST_APP_NO_PAUSE%"=="1" exit /b %APP_EXIT%
pause
exit /b %APP_EXIT%

:fail
echo.
echo Rust app launch failed.
echo.
if /I "%RUST_APP_NO_PAUSE%"=="1" exit /b 1
pause
exit /b 1

:ensure_dev_server
echo [Rust App] Starting frontend dev server in a separate window...

where npm >nul 2>&1
if errorlevel 1 (
    echo ERROR: npm was not found in PATH. Cannot start frontend dev server.
    echo Install Node.js or run with: set RUST_APP_SKIP_DEV_SERVER=1
    exit /b 1
)

REM Invoke Vite directly via node to avoid npm spawning a bash console.
start "Rust Frontend Dev Server" cmd /k "cd /d ""%~dp0frontend"" && node node_modules\vite\bin\vite.js"

echo [Rust App] Waiting for dev server on 127.0.0.1:5173...
set "TRIES=0"
:wait_for_server
set /a TRIES+=1
netstat -ano | findstr /R /C:":5173 .*LISTENING" >nul
if not errorlevel 1 (
    echo [Rust App] Dev server is ready.
    exit /b 0
)
if %TRIES% GEQ 20 (
    echo ERROR: Dev server did not start on port 5173.
    echo Check the "Rust Frontend Dev Server" window for npm errors.
    exit /b 1
)
timeout /t 1 /nobreak >nul
goto :wait_for_server
