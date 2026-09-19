@echo off
setlocal enabledelayedexpansion
rem Copyright 2023 RobustMQ Team
rem
rem Licensed under the Apache License, Version 2.0 (the "License");
rem you may not use this file except in compliance with the License.
rem You may obtain a copy of the License at
rem
rem     http://www.apache.org/licenses/LICENSE-2.0
rem
rem Unless required by applicable law or agreed to in writing, software
rem distributed under the License is distributed on an "AS IS" BASIS,
rem WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
rem See the License for the specific language governing permissions and
rem limitations under the License.

rem Windows startup script for the RobustMQ broker server.
rem Mirrors bin/robust-server (the POSIX shell script) for Windows hosts.

set "WORKDIR=%~dp0"
set "ROOT=%WORKDIR%.."
set "BIN=%ROOT%\libs\broker-server.exe"
set "LOGDIR=%ROOT%\logs"
set "LOGFILE=%LOGDIR%\robustmq.log"

set "ACTION=%~1"
set "CONF=%~2"

if "%ACTION%"=="" goto usage
if /I "%ACTION%"=="start" goto args_ok
if /I "%ACTION%"=="stop"  goto args_ok
echo Invalid action: %ACTION%, optional: start, stop
goto usage

:args_ok
if "%CONF%"=="" set "CONF=%ROOT%\config\server.toml"
if /I "%ACTION%"=="start" goto start
if /I "%ACTION%"=="stop"  goto stop

:usage
echo Usage: %~nx0 ^<action^> [config_file]
echo   action: start ^| stop
echo   config_file: optional, default is config\server.toml
echo   example start: %~nx0 start config\server.toml
echo   example stop:  %~nx0 stop
exit /b 1

:start
if not exist "%BIN%" (
  echo ERROR: %BIN% not found
  exit /b 1
)
if not exist "%CONF%" (
  echo Config file not found: %CONF%
  exit /b 1
)
if not exist "%LOGDIR%" mkdir "%LOGDIR%"

rem Read http_port from config file, default to 58080 if not found.
set "HTTP_PORT="
for /f "tokens=1,* delims==" %%A in ('findstr /r /c:"^[ ]*http_port[ ]*=" "%CONF%" 2^>nul') do set "HTTP_PORT=%%B"
if defined HTTP_PORT set "HTTP_PORT=!HTTP_PORT: =!"
if not defined HTTP_PORT set "HTTP_PORT=58080"

rem Write runtime config.js so the frontend picks up the correct API port
rem without needing a rebuild (best effort; only when dist/ is present).
if not exist "%ROOT%\dist" goto skip_config_js
set "CONFIGJS=%ROOT%\dist\config.js"
> "!CONFIGJS!"  echo // Runtime configuration written by robust-server on startup.
>>"!CONFIGJS!"  echo // Do not edit manually; changes will be overwritten on next start.
>>"!CONFIGJS!"  echo window.__APP_CONFIG__ = {
>>"!CONFIGJS!"  echo   api: {
>>"!CONFIGJS!"  echo     port: !HTTP_PORT!
>>"!CONFIGJS!"  echo   }
>>"!CONFIGJS!"  echo };
echo Runtime config written: !CONFIGJS!
:skip_config_js

echo Config: %CONF%
echo HTTP API port: !HTTP_PORT!
echo Starting RobustMQ broker server...
start "RobustMQ Broker" /B "%BIN%" --conf="%CONF%" >>"%LOGFILE%" 2>&1

rem Give the process a moment to come up, then verify it is running.
ping -n 4 127.0.0.1 >nul 2>&1
tasklist /FI "IMAGENAME eq broker-server.exe" 2>nul | findstr /I "broker-server.exe" >nul
if errorlevel 1 (
  echo [FAIL] broker failed to start. Please check details in %LOGFILE%
  exit /b 1
)
echo [OK] RobustMQ broker started successfully.
echo Log file location: %LOGFILE%
echo To view logs:      type "%LOGFILE%"
echo To check status:   tasklist ^| findstr /I broker-server
exit /b 0

:stop
tasklist /FI "IMAGENAME eq broker-server.exe" 2>nul | findstr /I "broker-server.exe" >nul
if errorlevel 1 (
  echo No running processes found for broker-server.
  exit /b 0
)
echo Stopping RobustMQ broker server...
taskkill /F /IM broker-server.exe >nul 2>&1
ping -n 3 127.0.0.1 >nul 2>&1
tasklist /FI "IMAGENAME eq broker-server.exe" 2>nul | findstr /I "broker-server.exe" >nul
if errorlevel 1 (
  echo [OK] RobustMQ broker stopped successfully.
  echo Final logs available at: %LOGFILE%
  exit /b 0
)
echo [FAIL] broker stop failure. Check logs for details: %LOGFILE%
exit /b 1
