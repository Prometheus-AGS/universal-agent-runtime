@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0refresh-credentials.ps1" %*
exit /b %ERRORLEVEL%
