@echo off
REM SPDX-License-Identifier: MIT
REM Build the ClipVault Windows installer (Inno Setup 6).
REM Run from anywhere; resolves paths relative to this script.
setlocal
set "ROOT=%~dp0.."
set "ISCC=%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe"

if not exist "%ISCC%" (
  echo [!] Inno Setup 6 ISCC.exe not found at: %ISCC%
  echo     Install with: winget install --id JRSoftware.InnoSetup -e
  exit /b 1
)

echo [1/3] Building release executable...
pushd "%ROOT%"
cargo build --release
if errorlevel 1 ( echo [!] cargo build failed & exit /b 1 )

echo [2/3] Generating brand icon...
cargo run --example dump_icon --quiet
if errorlevel 1 ( echo [!] icon generation failed & exit /b 1 )

echo [3/3] Compiling installer...
"%ISCC%" "installer\ClipVault.iss"
if errorlevel 1 ( echo [!] ISCC failed & exit /b 1 )

echo.
echo [OK] Installer ready:
echo      %ROOT%\installer\Output\ClipVault-0.1.2-Setup.exe
popd
endlocal
