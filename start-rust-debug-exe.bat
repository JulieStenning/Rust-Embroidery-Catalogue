@echo off
REM -----------------------------------------------------------------------
REM start-rust-debug-exe.bat  —  Launch the built debug EXE with correct CWD
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

REM -----------------------------------------------------------------------
REM Sync the developer database from the project root into the debug output
REM tree so the exe always finds Data\Database\EmbroideryCatalogue.db next
REM to itself.  (cargo clean wipes target\debug, so this runs every launch.)
REM -----------------------------------------------------------------------
echo [Rust App] Syncing development database...

if not exist "Data\Database\EmbroideryCatalogue.db" (
    echo ERROR: Data\Database\EmbroideryCatalogue.db was not found at the project root.
    echo The app cannot start without its database.
    pause
    exit /b 1
)

if not exist "target\debug\Data\Database" mkdir "target\debug\Data\Database"
if errorlevel 1 (
    echo ERROR: Could not create target\debug\Data\Database.
    pause
    exit /b 1
)

copy /Y "Data\Database\EmbroideryCatalogue.db" "target\debug\Data\Database\EmbroideryCatalogue.db" >nul
if errorlevel 1 (
    echo ERROR: Failed to copy database to target\debug\Data\Database\EmbroideryCatalogue.db.
    pause
    exit /b 1
)

echo [Rust App] Database copied to target\debug\Data\Database\EmbroideryCatalogue.db

echo [Rust App] Launching target\debug\embroidery-catalogue.exe
"target\debug\embroidery-catalogue.exe"
set "APP_EXIT=%ERRORLEVEL%"

echo.
echo [Rust App] EXE exited with code: %APP_EXIT%
if not "%RUST_APP_NO_PAUSE%"=="1" pause
exit /b %APP_EXIT%