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

if [ -z "$VINTAGE_STORY_MODS" ]; then
    echo "VINTAGE_STORY_MODS is not defined."
    
    while true; do
        read -p "Paste the Vintage Story Mods path: " input

        if [ -n "$input" ]; then
            VINTAGE_STORY_MODS="$input"
            export VINTAGE_STORY_MODS
            break
        else
            echo "Invalid path."
        fi
    done
fi

MOD_LIST=""
echo "Type the Mods IDs. Type 'end' to finish."

while true; do
    read -p "Mod ID: " mod_id

    if [ "$mod_id" = "end" ]; then
        break
    fi

    if [ -z "$mod_id" ]; then
        echo "Invalid ID."
        continue
    fi

    if [ -n "$MOD_LIST" ]; then
        MOD_LIST="${MOD_LIST},${mod_id}"
    else
        MOD_LIST="${mod_id}"
    fi
done

"$EXECUTABLE" \
    --working-path "$VINTAGE_STORY" \
    --mods-path "$VINTAGE_STORY_MODS" \
    --generate-modpack $MOD_LIST \
    --ignore-game-update