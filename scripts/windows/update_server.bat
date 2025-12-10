@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"

set "EXECUTABLE=%SCRIPT_DIR%\vintagestory_updater_windows.exe"

if exist "%EXECUTABLE%" (
    set "VINTAGE_STORY=%SCRIPT_DIR%"
) else (
    if "%VINTAGE_STORY%"=="" exit /b 1
    set "EXECUTABLE=%VINTAGE_STORY%\vintagestory_updater_windows.exe"
)

if not exist "%EXECUTABLE%" exit /b 1

"%EXECUTABLE%" ^
    --ignore-folders "ClientData,ServerData" ^
    --ignore-files "run.bat,update_client.bat,update_server.bat,update_mods.bat,add_mods.bat,server-run.bat" ^
    --game-type "server" ^
    --working-path "%VINTAGE_STORY%"

endlocal