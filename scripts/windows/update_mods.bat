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
    --working-path "%VINTAGE_STORY%" ^
    --ignore-game-update

endlocal