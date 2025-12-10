@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"

set "EXECUTABLE=%SCRIPT_DIR%\vintagestory_updater_windows.exe"

if exist "%EXECUTABLE%" (
    set "VINTAGE_STORY=%SCRIPT_DIR%"
) else (
        if "%VINTAGE_STORY%"=="" (
        echo [ERROR] Cannot find vintagestory_updater_windows.exe
        pause
        exit /b 1
    )
    set "EXECUTABLE=%VINTAGE_STORY%\vintagestory_updater_windows.exe"
)

if not exist "%EXECUTABLE%" (
    echo [ERROR] Cannot find %EXECUTABLE%
    pause
    exit /b 1
)

if "%VINTAGE_STORY_MODS%"=="" (
:input_mods
    set /p input=Paste the Vintage Story Mods path: 
    if "%input%"=="" goto input_mods
    set "VINTAGE_STORY_MODS=%input%"
)

set "MOD_LIST="

:mod_loop
set /p mod_id=Mod ID: 
if "%mod_id%"=="end" goto done_mods
if "%mod_id%"=="" goto mod_loop

if "%MOD_LIST%"=="" (
    set "MOD_LIST=%mod_id%"
) else (
    set "MOD_LIST=%MOD_LIST%,%mod_id%"
)

goto mod_loop

:done_mods

"%EXECUTABLE%" ^
    --working-path "%VINTAGE_STORY%" ^
    --mods-path "%VINTAGE_STORY_MODS%" ^
    --generate-modpack "%MOD_LIST%" ^
    --ignore-game-update

echo.
echo Finished.
pause
endlocal
