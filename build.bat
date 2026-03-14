@echo off
:: ========================================
::   Tender Checker - Build Script
::   Compatible with all Windows versions
:: ========================================

setlocal enabledelayedexpansion

echo.
echo ========================================
echo   Tender Checker - Build Script v5
echo ========================================
echo.

:: Check admin
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Administrator privileges required!
    echo.
    echo Please right-click this file and select:
    echo "Run as Administrator"
    echo.
    pause
    exit /b 1
)

echo [OK] Administrator privileges confirmed
echo.

:: Set install path
set "DEFAULT_PATH=C:\DevTools"
set "INSTALL_PATH="

echo Configure install path for Rust/Cargo
echo Default: %DEFAULT_PATH%
echo Press Enter for default, or type custom path (e.g., D:\DevTools)
echo.
set /p INSTALL_PATH="Install path: "

if "%INSTALL_PATH%"=="" set "INSTALL_PATH=%DEFAULT_PATH%"

echo.
echo Will install to: %INSTALL_PATH%
echo.

:: Create directories
if not exist "%INSTALL_PATH%" (
    mkdir "%INSTALL_PATH%"
    echo [OK] Created directory
)

set "RUST_PATH=%INSTALL_PATH%\rust"
set "CARGO_PATH=%INSTALL_PATH%\cargo"

if not exist "%RUST_PATH%" mkdir "%RUST_PATH%"
if not exist "%CARGO_PATH%" mkdir "%CARGO_PATH%"

:: Set environment variables
setx RUSTUP_HOME "%RUST_PATH%" >nul 2>&1
setx CARGO_HOME "%CARGO_PATH%" >nul 2>&1
set "RUSTUP_HOME=%RUST_PATH%"
set "CARGO_HOME=%CARGO_PATH%"
set "PATH=%CARGO_PATH%\bin;%PATH%"

echo [OK] Environment variables set
echo.

:: ========================================
:: Check Node.js
:: ========================================
echo ========================================
echo   Step 1/3: Check Node.js
echo ========================================
echo.

where node >nul 2>&1
if %errorLevel% equ 0 (
    for /f "tokens=*" %%i in ('node -v 2^>nul') do set NODE_VER=%%i
    echo [OK] Node.js installed: !NODE_VER!
) else (
    echo [!] Node.js NOT installed
    echo.
    echo Please install Node.js from: https://nodejs.org/
    echo Recommended: LTS version
    echo.
    pause
    exit /b 0
)

echo.

:: ========================================
:: Check Rust
:: ========================================
echo ========================================
echo   Step 2/3: Check Rust
echo ========================================
echo.

where rustc >nul 2>&1
if %errorLevel% equ 0 (
    for /f "tokens=*" %%i in ('rustc --version 2^>nul') do set RUST_VER=%%i
    echo [OK] Rust installed: !RUST_VER!
) else (
    echo [!] Rust NOT installed
    echo.
    echo Please install Rust:
    echo   1. Visit https://rustup.rs/
    echo   2. Download and run rustup-init.exe
    echo   3. Run this script again after installation
    echo.
    echo Or install via command line:
    echo   winget install Rustlang.Rustup
    echo.
    pause
    exit /b 0
)

echo.

:: ========================================
:: Check VS Build Tools
:: ========================================
echo ========================================
echo   Step 3/3: Check VS Build Tools
echo ========================================
echo.

set "HAS_VS=0"

:: Check for VS 2022
if exist "%ProgramFiles%\Microsoft Visual Studio\2022\BuildTools\" set "HAS_VS=1"
if exist "%ProgramFiles%\Microsoft Visual Studio\2022\Community\" set "HAS_VS=1"
if exist "%ProgramFiles%\Microsoft Visual Studio\2022\Professional\" set "HAS_VS=1"
if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\" set "HAS_VS=1"

:: Check for VS 2019
if exist "%ProgramFiles%\Microsoft Visual Studio\2019\BuildTools\" set "HAS_VS=1"
if exist "%ProgramFiles%\Microsoft Visual Studio\2019\Community\" set "HAS_VS=1"
if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2019\BuildTools\" set "HAS_VS=1"

if "%HAS_VS%"=="1" (
    echo [OK] VS Build Tools detected
) else (
    echo [!] VS Build Tools NOT installed
    echo.
    echo VS Build Tools is required to compile Tauri apps.
    echo Note: Can ONLY install to C: drive, requires ~6GB
    echo.
    set /p INSTALL_VS="Install VS Build Tools? (y/n): "
    
    if /i "!INSTALL_VS!"=="y" (
        echo.
        echo Installing VS Build Tools...
        echo This will take 15-30 minutes...
        echo.
        winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
        echo.
        echo [OK] Installation complete
        echo Please run this script again.
        pause
        exit /b 0
    ) else (
        echo.
        echo VS Build Tools is required. Cannot continue.
        pause
        exit /b 0
    )
)

echo.

:: ========================================
:: Build Project
:: ========================================
echo ========================================
echo   Building Project
echo ========================================
echo.

cd /d "%~dp0"

echo Project directory: %cd%
echo.

if not exist "package.json" (
    echo [ERROR] package.json not found
    echo Please run script from project root directory
    pause
    exit /b 1
)

:: Install npm dependencies
echo Installing npm dependencies...
echo.

call npm install
if %errorLevel% neq 0 (
    echo.
    echo [ERROR] npm install failed
    echo.
    echo Possible fixes:
    echo   1. Check network connection
    echo   2. Try: npm config set registry https://registry.npmmirror.com
    echo   3. Upgrade Node.js to LTS version
    pause
    exit /b 1
)

echo.
echo [OK] npm dependencies installed
echo.

:: Build Tauri
echo Building Tauri application...
echo First build takes 5-15 minutes, please wait...
echo.

call npm run tauri:build
if %errorLevel% neq 0 (
    echo.
    echo ========================================
    echo   Build Failed!
    echo ========================================
    echo.
    echo Common issues:
    echo   1. VS Build Tools not installed
    echo   2. Rust version outdated - run: rustup update
    echo   3. Network issue - need to download Rust crates
    echo.
    pause
    exit /b 1
)

echo.
echo ========================================
echo   Build Success!
echo ========================================
echo.
echo Output files:
echo   EXE: %cd%\src-tauri\target\release\tender-checker.exe
echo   MSI: %cd%\src-tauri\target\release\bundle\msi\
echo.
echo Environment variables saved. Restart PowerShell to take effect.
echo.

pause
