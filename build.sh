#!/bin/sh
if [ -z "$VINTAGE_STORY" ] || [ ! -d "$VINTAGE_STORY" ]; then
    echo "VINTAGE_STORY undefined or invalid directory."
    read -p "Vintage Story mods path: " VINTAGE_STORY

    if [ ! -d "$VINTAGE_STORY" ]; then
        mkdir -p "$VINTAGE_STORY"
    fi
fi

cargo build --release

OS="$(uname -s)"

# Auto place the build to vintage story folder
if [[ "$OS" == "Linux" ]]; then
    cp -r ./target/release/vintagestory_updater "$VINTAGE_STORY/vintagestory_updater_linux"
elif [[ "$OS" == "MINGW"* || "$OS" == "CYGWIN"* || "$OS" == "MSYS"* ]]; then
    cp -r ./target/release/vintagestory_updater.exe "$VINTAGE_STORY/vintagestory_updater_windows.exe"
else
    echo "Not supported: $OS"
    exit 1
fi