@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-test.ps1"
echo.
echo Please read install-test.log for the installation result.
pause
