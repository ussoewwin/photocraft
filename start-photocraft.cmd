@echo off
rem PhotoCraft launcher: build and run the desktop app straight from this repository.
rem Everything resolves from this script's own folder; nothing machine-specific is hardcoded.

setlocal
cd /d "%~dp0"

set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
if exist "%CARGO%" goto run
where cargo >nul 2>nul
if %errorlevel%==0 (
  set "CARGO=cargo"
  goto run
)

echo [PhotoCraft] Rust toolchain not found.
echo.
echo This repository builds from source, which needs Rust via rustup
echo (stable, MSVC host; minimum supported version 1.95).
echo Install it from https://rustup.rs and then run this script again.
echo A Visual Studio 2022 C++ toolset must also be present (already found on this machine).
echo.
pause
exit /b 1

:run
echo [PhotoCraft] Building and starting the desktop app (cargo run --release -p photocraft).
echo [PhotoCraft] The first run compiles the whole workspace and may take a long time.
"%CARGO%" run --release -p photocraft
if errorlevel 1 (
  echo.
  echo [PhotoCraft] Build or launch failed. Check the errors above.
  pause
)
endlocal
