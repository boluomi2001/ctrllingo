@echo off
cd /d "%~dp0"
set "CTRLLINGO_NODE=%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe"
if not exist "%CTRLLINGO_NODE%" set "CTRLLINGO_NODE=node"
"%CTRLLINGO_NODE%" node_modules\vite\bin\vite.js --host 127.0.0.1 --port 5173 --open
pause
