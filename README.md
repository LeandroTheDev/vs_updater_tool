# Vintage Story Updater Tool
- Linux Dependencies: ``wget``, ``tar``, ``unzip`` (Some distros does not have wget/unzip by default)
- Windows Dependencies: ``Invoke-WebRequest``, ``Expand-Archive`` (Generally a fresh windows install contains all this features)

## Compatibility
- Windows (Servers Only)
- Linux

### Usage
- Windows users: download the .exe, put it inside the your vintage story server folder, double click it to open, and should download it normally
- Linux users: download the executable, put it inside your vintage story folder, open terminal inside that folder: ``./vs_updater_tool -- --game-type server`` if is a server or ``./vs_updater_tool -- --game-type client`` if is a client, and it should download it for you automatically

``Linux Users``
Linux users or even windows can use the ``update.sh`` script to automatically update the game client

## ``IMPORTANT``
- Closing the vs updater tool before completing might result in files lost
```
The folder .temp already exists, probably the updater tool exited before completing.
Do you want to delete it? (y,N):
```
- Do not ignore this message, check the folder .temp inside ``working-path`` and restore files if needed

## Customization
- ignore-folders: ``--ignore-folders ServerData,ServerData2``, (Does not accept recursive)
- > This will save the folders to .temp folder, and will be replaced after updating the game (Does not accept recursive folders like ServerData/Mods)
- ignore-files: ``--ignore-files start-server.sh,run.sh``, (Does not accept recursive)
- > This will save the files to .temp folder, and will be replaced after updating the game
- working-path: ``--working-path /home/user/vintagestory/``
- > Currently vintagestory folder, if not set it will pickup from the system variable: ``VINTAGE_STORY`` or if also not exist it will use the same folder from executable
- game-type: ``--game-type server`` or ``--game-type client``
- > Select the game type, only a server or entire game as client
- ignore-game-update: ``--ignore-mod-update``
- > Ignore update of the game
- ignore-mod-update: ``--ignore-mod-update``
- > Ignore mods update
- mods-path: ``--mods-path /home/user/vintagestory/ServerData/Mods/``
- > Currently mods path to be updated, required if you are updating mods
- force-url: ``--force-url "https://cdn.vintagestory.at/gamefiles/unstable/vs_server_linux-x64_1.21.0-rc.4.tar.gz"``
- > Force update the game to a specific download url, used for downloading from other sources or secret versions
- no-pre-mods: ``--no-pre-mods``
- > Ignore unstable versions of mods
- no-pre: ``--no-pre``
- > Ignore unstable versions of vintage story

## Mod Update
To automatically update the mods you will need to get the id from the mod in vs database, the easy way to get the id is to go to the mod page: ``https://mods.vintagestory.at/rpgoverlay``, the ``rpgoverlay`` is the mod id, copy that and go to ``mods-path`` and create a new folder for example: ``rpgoverlay_1.0.0`` and create a new file inside that folder: ``modid.txt`` paste the mod id inside the file and add a 0 on next line, mods should automatically update when running the executable
```
rpgoverlay
0
```
- The ``modid.txt`` contains: modid and fileid, must be exactly that way

## Examples
Full example: ``./vs_updater_tool -- --ignore-folders ServerData,ServerData2 --ignore-files start-server.sh,run.sh --working-path /home/user/vintagestory/ --game-type server --mods-path /home/user/vintagestory/ServerData/Mods/``

No mods example: ``./vs_updater_tool -- --ignore-folders ServerData,ServerData2 --ignore-files start-server.sh,run.sh --working-path /home/user/vintagestory/ --game-type server --ignore-mod-update``

Only mods example: ``./vs_updater_tool -- --ignore-folders ServerData,ServerData2 --ignore-files start-server.sh,run.sh --working-path /home/user/vintagestory/ --game-type server --ignore-game-update --mods-path /home/user/vintagestory/ServerData/Mods/``