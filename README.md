# Evolved DragonFable Launcher
This is a launcher with quality-of-life features for DragonFable.


## Building
The launcher consists of 2 parts:
1. A local caching proxy server, written in rust (`/core`)
2. An Electron frontend (`/ui`)


Before building, you need to:
1. Ensure `rust`, `cargo`, and `npm` are installed
2. Run `npm install` in the `/ui` folder
3. Create the folder `/ui/plugins`, and put the flash player library there. I cannot redistribute this library because of its license, so it's not included in this repo
The flash player library should be named:
- `libpepflashplayer.so` for Linux
- `pepflashplayer.dll` for Windows
- `PepperFlashPlayer` for MacOS
4. Create the folder `/assets`, and put the app's icon there


To build the app, you need to:
1. Build the Electron app at `/ui`
2. Rename the executable `evolved-dragonfable-launcher` to `ui` (on windows, rename `evolved-dragonfable-launcher.exe` to `ui.exe`)
3. Build the proxy server at `/core`
4. Move the proxy server binary to Electron's output folder (the same folder as the `ui` executable)
5. To run the app, open `evolved-dragonfable-launcher`


The scripts `run.sh` and `build.sh` are supplied for running/building automatically. The output of `build.sh` will be put at `/out`.
