@echo off
setlocal EnableExtensions
title Embroidery Catalogue Uninstaller

echo ============================================
echo   Embroidery Catalogue Uninstaller
echo ============================================
echo.

rem ------------------------------------------------------------------
rem 1. Close any running instance (all name variants) and wait for locks
rem ------------------------------------------------------------------
echo [1/5] Closing any running instances...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-Process | Where-Object { $_.ProcessName -like '*mbroidery*' } | Stop-Process -Force -ErrorAction SilentlyContinue"
timeout /t 2 /nobreak >nul

rem ------------------------------------------------------------------
rem 2. Unregister from 'Add/Remove Programs' via the registered
rem    uninstaller (Tauri v2 uses an MSI/Windows Installer package)
rem ------------------------------------------------------------------
echo [2/5] Unregistering from 'Add/Remove Programs'...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$paths = @('HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'); $app = Get-ItemProperty -Path $paths -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like 'Embroidery*' -and $_.DisplayName -like '*Catalogue*' } | Select-Object -First 1; if ($app) { Write-Host ('   Found: ' + $app.DisplayName); $g = $app.PSObject.Properties.Name; if ($g -contains 'QuietUninstallString') { cmd /c $app.QuietUninstallString } elseif ($g -contains 'UninstallString') { cmd /c $app.UninstallString } elseif ($g -contains 'ProductCode') { Start-Process msiexec.exe -ArgumentList @('/x', $app.ProductCode, '/qn', '/norestart') -Wait } else { Write-Host '   No uninstall command available in registration.' } } else { Write-Host '   No Add/Remove Programs registration found for Embroidery Catalogue.' }"
timeout /t 2 /nobreak >nul

rem ------------------------------------------------------------------
rem 3. Remove user application data folder (with retries to handle locks)
rem    %APPDATA%\EmbroideryCatalogue  and  %APPDATA%\com.embroidery-catalogue
rem ------------------------------------------------------------------
echo [3/5] Removing user data folders...
for /L %%i in (1,1,3) do (
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Remove-Item -LiteralPath ($env:APPDATA + '\EmbroideryCatalogue'), ($env:APPDATA + '\com.embroidery-catalogue') -Recurse -Force -ErrorAction SilentlyContinue"
    timeout /t 1 /nobreak >nul
)

if exist "%APPDATA%\EmbroideryCatalogue" (
    echo    WARNING: Could not fully remove "%APPDATA%\EmbroideryCatalogue" - some files may still be in use.
) else (
    echo    Removed: "%APPDATA%\EmbroideryCatalogue"
)

if exist "%APPDATA%\com.embroidery-catalogue" (
    echo    WARNING: Could not fully remove "%APPDATA%\com.embroidery-catalogue" - some files may still be in use.
) else (
    echo    Removed: "%APPDATA%\com.embroidery-catalogue"
)

rem ------------------------------------------------------------------
rem 4. Remove application installation directories (fallback cleanup)
rem ------------------------------------------------------------------
echo [4/5] Removing application files...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$dirs = @(\"$env:LOCALAPPDATA\Embroidery Catalogue\", \"$env:LOCALAPPDATA\EmbroideryCatalogue\", \"$env:ProgramFiles\Embroidery Catalogue\", \"$env:ProgramFiles\EmbroideryCatalogue\", \"${env:ProgramFiles(x86)}\Embroidery Catalogue\", \"${env:ProgramFiles(x86)}\EmbroideryCatalogue\"); foreach ($d in $dirs) { if (Test-Path -LiteralPath $d) { Remove-Item -LiteralPath $d -Recurse -Force -ErrorAction SilentlyContinue; Write-Host ('   Removing: ' + $d) } }"

rem ------------------------------------------------------------------
rem 5. Remove Start Menu and Desktop shortcuts
rem ------------------------------------------------------------------
echo [5/5] Removing shortcuts...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$lnks = @(\"$env:PUBLIC\Desktop\Embroidery Catalogue.lnk\", \"$env:USERPROFILE\Desktop\Embroidery Catalogue.lnk\", \"$env:PUBLIC\Desktop\EmbroideryCatalogue.lnk\", \"$env:USERPROFILE\Desktop\EmbroideryCatalogue.lnk\", \"$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Embroidery Catalogue\"); foreach ($f in $lnks) { if (Test-Path -LiteralPath $f) { Remove-Item -LiteralPath $f -Recurse -Force -ErrorAction SilentlyContinue; Write-Host ('   Removing: ' + $f) } }"

rem ------------------------------------------------------------------
rem Done
rem ------------------------------------------------------------------
echo.
echo Embroidery Catalogue has been removed from this computer.
echo.
pause
endlocal