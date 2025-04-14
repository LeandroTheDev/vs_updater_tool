# Vintage Story Updater Tool
- Linux Dependencies: ``wget``, ``tar``
- Windows Dependencies: ``Invoke-WebRequest``, ``curl``, 

### Usage
- Windows users: download the .exe, put it inside the your vintage story server folder, double click it to open, and should download it normally
- Linux users: download the executable, put it inside your vintage story folder, open terminal inside that folder: ``./vs_updater_tool -- --game-type server`` if is a server or ``./vs_updater_tool -- --game-type client`` if is a client, and it should download it for you automatically

## Customization
- ignore-folders: ``/home/user/vintagestory/ServerData,/home/user/vintagestory/ServerData2``
- > This will save the folders inside the .temp folder, and will be replaced after updating the game
- ignore-files: ``/home/user/vintagestory/start-server.sh,/home/user/vintagestory/run.sh``,
- > This will save the files inside the .temp folder, and will be replaced after updating the game
- working-path: ``/home/user/vintagestory/``
- > Currently vintagestory folder
- game-type: ``server`` or ``client``
- > Select the game type, only a server or entire game as client

Full example: ``./vs_updater_tool -- --ignore-folders /home/user/vintagestory/ServerData,/home/user/vintagestory/ServerData2 --ignore-files /home/user/vintagestory/start-server.sh,/home/user/vintagestory/run.sh --working-path /home/user/vintagestory/ --game-type client``