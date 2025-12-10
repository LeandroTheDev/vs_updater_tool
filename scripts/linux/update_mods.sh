#!/bin/bash
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

OS="$(uname -s)"

if [[ "$OS" == "Linux" ]]; then
    EXECUTABLE="$SCRIPT_DIR/vintagestory_updater_linux"
elif [[ "$OS" == "MINGW"* || "$OS" == "CYGWIN"* || "$OS" == "MSYS"* ]]; then
    EXECUTABLE="$SCRIPT_DIR/vintagestory_updater_windows.exe"
else
    echo "System not supported: $OS"
    exit 1
fi

if [[ -f "$EXECUTABLE" ]]; then
    VINTAGE_STORY="$SCRIPT_DIR"
else
    if [[ -z "$VINTAGE_STORY" ]]; then
        echo "[ERROR] Cannot find vintagestory_updater tool"
        exit 1
    fi
    
    if [[ "$OS" == "Linux" ]]; then
        EXECUTABLE="$VINTAGE_STORY/vintagestory_updater_linux"
    else
        EXECUTABLE="$VINTAGE_STORY/vintagestory_updater_windows.exe"
    fi
fi

if [[ ! -f "$EXECUTABLE" ]]; then
    echo "[ERROR] Cannot find vintagestory_updater tool"
    exit 1
fi

"$EXECUTABLE" \
    --working-path "$VINTAGE_STORY" \
    --ignore-game-update