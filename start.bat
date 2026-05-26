@echo off
REM -----------------------------------------------------------------------
REM start.bat  —  Launch the Embroidery Catalogue local web application
REM
REM  Dev machine:  uses .venv\  (created by  python -m venv .venv)
REM  SD card:      uses venv\   (created automatically by setup.bat)
REM
REM  Optional overrides before calling start.bat:
REM    set APP_PORT=8003
REM    set DATABASE_URL=sqlite:///D:/path/to/data/database/catalogue_dev.db
REM    set EMBROIDERY_DISABLE_ERROR_PAUSE=1
REM -----------------------------------------------------------------------

REM Change to the directory this script lives in
cd /d "%~dp0"
if not exist "logs" mkdir "logs"
set "STARTUP_LOG=%CD%\logs\startup-error.log"
(
    echo ==== [%date% %time%] start.bat launch ====
    echo Working directory: %CD%
) > "%STARTUP_LOG%"

REM --- 0. Stop any existing instance before starting ---
call "%~dp0stop.bat"

set "APP_ENV="
set "DEFAULT_DB_URL="
set "DB_URL_OVERRIDDEN=0"

if defined DATABASE_URL set "DB_URL_OVERRIDDEN=1"

REM --- 1. Create the managed data directories if they don't exist ---
if not exist "data" mkdir data
if not exist "data\database" mkdir "data\database"
if not exist "data\MachineEmbroideryDesigns" mkdir "data\MachineEmbroideryDesigns"

REM --- 2. Select / prepare the virtual environment ---
if exist ".venv\Scripts\activate.bat" (
    REM Developer machine — use existing .venv. Reload is opt-in because it can serve stale code on Windows.
    call .venv\Scripts\activate.bat
    set "UVICORN_RELOAD="
    if /I "%APP_RELOAD%"=="1" set "UVICORN_RELOAD=--reload"
    set "APP_ENV=development"
    if not defined APP_PORT set "APP_PORT=8003"
    set "DEFAULT_DB_URL=sqlite:///%CD:/=/%/data/database/catalogue_dev.db"
) else (
    REM SD card / portable mode — bootstrap on first run
    if not exist "venv\Scripts\python.exe" (
        echo First run: running setup.bat ...
        call "%~dp0setup.bat"
        if errorlevel 1 call :fail_startup "first-time setup" 1
    )
    call "%~dp0venv\Scripts\activate.bat"
    set UVICORN_RELOAD=
    set "APP_ENV=portable"
    if not defined APP_PORT set "APP_PORT=8002"
    set "DEFAULT_DB_URL=sqlite:///%CD:/=/%/data/database/catalogue.db"
)

if not defined DATABASE_URL set "DATABASE_URL=%DEFAULT_DB_URL%"

REM --- 3. Development first-run DB bootstrap ---
if /I "%APP_ENV%"=="development" (
    if "%DB_URL_OVERRIDDEN%"=="0" (
        if not exist "data\database\catalogue_dev.db" (
            if exist "data\database\catalogue.db" (
                echo Initializing development database from data\database\catalogue.db ...
                copy /Y "data\database\catalogue.db" "data\database\catalogue_dev.db" >nul
            ) else if exist "data\catalogue_dev.db" (
                echo Migrating legacy development database from data\catalogue_dev.db ...
                copy /Y "data\catalogue_dev.db" "data\database\catalogue_dev.db" >nul
            ) else if exist "data\catalogue.db" (
                echo Migrating legacy database from data\catalogue.db ...
                copy /Y "data\catalogue.db" "data\database\catalogue_dev.db" >nul
            ) else (
                echo WARNING: no existing catalogue database was found to clone.
                echo          A new empty development database will be created by migrations.
            )
        )
    )
)

REM --- 4. Prepare the database ---
echo Environment: %APP_ENV%
echo Database: %DATABASE_URL%
echo Preparing database schema...
>> "%STARTUP_LOG%" echo Environment: %APP_ENV%
>> "%STARTUP_LOG%" echo Database: %DATABASE_URL%
set "STEP_LOG=%TEMP%\embroidery_start_%RANDOM%.log"
python -c "from src.database import bootstrap_database; print(f'Database setup: {bootstrap_database()}')" > "%STEP_LOG%" 2>&1
set "STEP_EXIT=%ERRORLEVEL%"
type "%STEP_LOG%"
>> "%STARTUP_LOG%" type "%STEP_LOG%"
del "%STEP_LOG%" >nul 2>&1
if not "%STEP_EXIT%"=="0" call :fail_startup "database preparation" %STEP_EXIT%

REM --- 5. Open browser after a short delay (unless suppressed for tests/automation) ---
set "SUPPRESS_BROWSER_OPEN="
if defined PYTEST_CURRENT_TEST set "SUPPRESS_BROWSER_OPEN=1"
if /I "%EMBROIDERY_DISABLE_EXTERNAL_OPEN%"=="1" set "SUPPRESS_BROWSER_OPEN=1"
if /I "%EMBROIDERY_DISABLE_EXTERNAL_OPEN%"=="true" set "SUPPRESS_BROWSER_OPEN=1"
if /I "%EMBROIDERY_DISABLE_EXTERNAL_OPEN%"=="yes" set "SUPPRESS_BROWSER_OPEN=1"
if /I "%EMBROIDERY_DISABLE_EXTERNAL_OPEN%"=="on" set "SUPPRESS_BROWSER_OPEN=1"
if "%SUPPRESS_BROWSER_OPEN%"=="1" (
    echo Browser auto-open suppressed.
) else (
    start "" /B cmd /C "ping 127.0.0.1 -n 3 >nul && start http://localhost:%APP_PORT%"
)

REM --- 6. Start the FastAPI application ---
echo Starting Embroidery Catalogue at http://localhost:%APP_PORT% ...
echo Log folder: %CD%\logs
echo Startup error log: %STARTUP_LOG%
>> "%STARTUP_LOG%" echo Starting Embroidery Catalogue at http://localhost:%APP_PORT% ...
python -m uvicorn src.main:app %UVICORN_RELOAD% --app-dir "%CD%" --host 127.0.0.1 --port %APP_PORT% --no-access-log --log-level info
set "START_EXIT=%ERRORLEVEL%"
if not "%START_EXIT%"=="0" call :fail_startup "web server startup" %START_EXIT%
exit /b %START_EXIT%

:fail_startup
echo.
echo ERROR: Startup failed during %~1.
echo Review the messages above for details.
echo Log folder: %CD%\logs
echo Startup error log: %STARTUP_LOG%
>> "%STARTUP_LOG%" echo ERROR: Startup failed during %~1.
>> "%STARTUP_LOG%" echo Review the messages above for details.
>> "%STARTUP_LOG%" echo Log folder: %CD%\logs
if defined PYTEST_CURRENT_TEST exit /b %~2
if /I "%EMBROIDERY_DISABLE_ERROR_PAUSE%"=="1" exit /b %~2
if /I "%EMBROIDERY_DISABLE_ERROR_PAUSE%"=="true" exit /b %~2
if /I "%EMBROIDERY_DISABLE_ERROR_PAUSE%"=="yes" exit /b %~2
if /I "%EMBROIDERY_DISABLE_ERROR_PAUSE%"=="on" exit /b %~2
pause
exit /b %~2
