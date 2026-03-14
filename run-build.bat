@echo off
:: 设置代码页为 UTF-8
chcp 65001 >nul 2>&1

title Tender Checker Build Script

echo.
echo ========================================
echo   Tender Checker - Build Script
echo ========================================
echo.
echo Starting PowerShell...
echo.

:: 使用 -NoExit 确保窗口不关闭
PowerShell -NoExit -ExecutionPolicy Bypass -Command "& { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; & '%~dp0build-windows.ps1' }"

echo.
echo PowerShell has exited.
echo.
pause
