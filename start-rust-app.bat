@echo off
REM -----------------------------------------------------------------------
REM start-rust-app.bat  —  Launch the Rust/Tauri desktop app in dev mode
REM
REM Optional overrides before calling start-rust-app.bat:
REM   set RUST_APP_SKIP_NPM_INSTALL=1
REM   set RUST_APP_NO_PAUSE=1
REM -----------------------------------------------------------------------

cd /d "%~dp0"

echo.
echo [Rust App] Starting from: %CD%
echo [Rust App] Mode: TAURI DEV (^"cargo tauri dev^"^). Use start-rust-debug-exe.bat for built EXE tests.

where cargo >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust cargo was not found in PATH.
    echo Install Rust from https://rustup.rs and retry.
    goto :fail
)

cargo tauri -V >nul 2>&1
if errorlevel 1 (
    if exist "%USERPROFILE%\.cargo\bin\cargo-tauri.exe" (
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
        cargo tauri -V >nul 2>&1
    )
)

cargo tauri -V >nul 2>&1
if errorlevel 1 (
    echo ERROR: Tauri CLI is not available for cargo.
    echo Run: cargo install tauri-cli --locked
    echo Then open a new terminal and run this script again.
    goto :fail
)

if not exist "frontend\package.json" (
    echo ERROR: frontend\package.json was not found.
    echo Ensure this script is run from the repository root.
    goto :fail
)

if /I not "%RUST_APP_SKIP_NPM_INSTALL%"=="1" (
    if not exist "frontend\node_modules" (
        echo [Rust App] Installing frontend dependencies...
        pushd frontend
        call npm install
        if errorlevel 1 (
            popd
            echo ERROR: npm install failed.
            goto :fail
        )
        popd
    )
)

REM Start the Vite dev server in a separate window to avoid console-allocation
REM exhaustion (Git Bash max 32 consoles).  Then wait for port 5173 before
REM launching Tauri so it can connect to devUrl immediately.
REM
REM We invoke Vite directly via node rather than through "npm run dev" so that
REM npm does not spawn an intermediate bash shell (which consumes a console slot).
echo [Rust App] Starting Vite dev server in a separate window...
start "Rust Frontend Dev Server" cmd /k "cd /d ""%~dp0frontend"" && node node_modules\vite\bin\vite.js"

echo [Rust App] Waiting for Vite dev server on 127.0.0.1:5173...
set "TRIES=0"
:wait_for_server
set /a TRIES+=1
netstat -ano | findstr /R /C:":5173 .*LISTENING" >nul
if not errorlevel 1 (
    echo [Rust App] Vite dev server is ready.
    goto :launch_tauri
)
if %TRIES% GEQ 30 (
    echo ERROR: Vite dev server did not start on port 5173 within 30 seconds.
    echo Check the "Rust Frontend Dev Server" window for npm errors.
    goto :fail
)
timeout /t 1 /nobreak >nul
goto :wait_for_server

:launch_tauri
echo [Rust App] Launching Tauri dev app...
cargo tauri dev --no-watch
set "APP_EXIT=%ERRORLEVEL%"
if not "%APP_EXIT%"=="0" goto :fail_with_code
exit /b 0

:fail_with_code
echo.
echo Rust app launch failed. Exit code: %APP_EXIT%
echo.
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
