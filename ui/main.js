const { app, BrowserWindow, Menu, MenuItem, shell } = require("electron")
const {join, sep, isAbsolute, resolve, dirname} = require("path")
const express = require("express")
const {existsSync, mkdirSync, readdirSync} = require("fs");
const {copySync} = require("fs-extra/lib/copy");
const {homedir} = require("os");

function mkDirByPathSync(targetDir, { isRelativeToScript = false } = {}) {
    const initDir = isAbsolute(targetDir) ? sep : '';
    const baseDir = isRelativeToScript ? __dirname : '.';

    return targetDir.split(sep).reduce((parentDir, childDir) => {
        const curDir = resolve(baseDir, parentDir, childDir);
        try {
            mkdirSync(curDir);
        } catch (err) {
            if (err.code === 'EEXIST') { // curDir already exists!
                return curDir;
            }

            // To avoid `EISDIR` error on Mac and `EACCES`-->`ENOENT` and `EPERM` on Windows.
            if (err.code === 'ENOENT') { // Throw the original parentDir error on curDir `ENOENT` failure.
                throw new Error(`EACCES: permission denied, mkdir '${parentDir}'`);
            }

            const caughtErr = ['EACCES', 'EPERM', 'EISDIR'].indexOf(err.code) > -1;
            if (!caughtErr || caughtErr && curDir === resolve(targetDir)) {
                throw err; // Throw if it's just the last created dir.
            }
        }

        return curDir;
    }, initDir);
}

const writable_root_path = join(app.getPath("userData"), "Pepper Data/Shockwave Flash/WritableRoot");
if (!existsSync(writable_root_path)) {
    const external_apps_writable_root_paths = [];

    // Try to copy data from Flash Player Standalone
    let flash_player_dir;
    if (process.platform === "linux") {
        flash_player_dir = join(homedir(), ".macromedia/Flash_Player");
        external_apps_writable_root_paths.push(flash_player_dir, );
    }
    else if (process.platform === "win"){
        flash_player_dir = join(process.env.APPDATA, "Macromedia/Flash Player");
        external_apps_writable_root_paths.push(flash_player_dir, );
    }

    // Try to copy data from the official launcher
    external_apps_writable_root_paths.push(writable_root_path.replace("evolved-dragonfable-launcher", "Artix Game Launcher"));

    let user_pref_dir = null;
    let object_dir_id = null;
    for (const writable_root_dir of external_apps_writable_root_paths) {
        const shared_objects_dir = join(writable_root_dir, "#SharedObjects");
        if (!existsSync(shared_objects_dir)) {
            continue;
        }

        let found = false;
        for (const object_dir_name of readdirSync(shared_objects_dir)) {
            for (const domain of ["play.dragonfable.com", "dragonlord.battleon.com", "dragonfable.battleon.com"]) {
                const object_path = join(shared_objects_dir, object_dir_name, domain);
                if (existsSync(object_path)) {
                    object_dir_id = object_dir_name;
                    user_pref_dir = object_path;
                    found = true;
                    break;
                }
            }
            if (found) {
                break;
            }
        }
        if (found) {
            break;
        }
    }

    if (user_pref_dir && object_dir_id) {
        let target_dir = join(writable_root_path, "#SharedObjects", object_dir_id, "127.0.0.1");
        mkDirByPathSync(dirname(target_dir));
        copySync(user_pref_dir, target_dir);
    }
}


let pluginName;

if (process.platform === "linux") {
  pluginName = "plugins/libpepflashplayer.so";
}
else if (process.platform === "win"){
  pluginName = "plugins/pepflashplayer.dll";
}
else {
  pluginName = "plugins/PepperFlashPlayer";
}

app.commandLine.appendSwitch("ppapi-flash-path", join(__dirname, pluginName))
app.commandLine.appendSwitch("ppapi-flash-version", "32.0.0.371");

const createWindow = () => {
    const win = new BrowserWindow({
        webPreferences: {
            plugins: true,
            nodeIntegration: false,
        },
        autoHideMenuBar: true,
    })

    win.webContents.on("new-window", function(e, url) {
        e.preventDefault();
        shell.openExternal(url);
    });

    win.loadFile("index.html")

    const app = express()
    app.use(express.json())

    app.post("/character", (req, res) => {
        win.webContents.executeJavaScript("receiveCharacterData(\"" + encodeURIComponent(JSON.stringify(req.body)) + "\")").then(_ => {})
        res.send("")
    })

    app.listen(39621, "127.0.0.1", () => {})
}

app.whenReady().then(() => {
    createWindow();

    app.on("activate", () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow()
    })

    const menu = new Menu()
    menu.append(new MenuItem({
        label: "Electron",
        submenu: [
            {
                label: "Fullscreen",
                role: "togglefullscreen",
            },
            {
                label: "Developer Tools",
                role: "toggleDevTools",
            }
        ]
    }))

    Menu.setApplicationMenu(menu)
})

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit()
})
