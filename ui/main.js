const { app, BrowserWindow, Menu, MenuItem } = require("electron")
const {join, basename} = require("path")
const express = require("express")

let pluginName;

if (process.platform === "linux") {
    pluginName = "plugins/libpepflashplayer.so";
}
else {
    pluginName = "plugins/pepflashplayer.dll";
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
                role: "Fullscreen",
                accelerator: "Alt+Enter",
                click: () => {
                    let win = BrowserWindow.getFocusedWindow();
                    win.setFullScreen(!win.isFullScreen());
                }
            },
            {
                role: "Developer Tools",
                accelerator: "Ctrl+Shift+I",
                click: () => {
                    BrowserWindow.getFocusedWindow().webContents.toggleDevTools();
                }
            }
        ]
    }))

    Menu.setApplicationMenu(menu)
})

app.on("window-all-closed", () => {
    if (process.platform !== "darwin") app.quit()
})
