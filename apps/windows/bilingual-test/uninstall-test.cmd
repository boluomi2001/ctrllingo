@echo off
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall-test.ps1"
echo.
echo Please read uninstall-test.log for the result. Restart Windows before removing the test folder.
pause
