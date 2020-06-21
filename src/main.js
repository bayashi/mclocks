const isDebug = process.argv.includes('--debug');

const mConfig = require('./lib/config');
const mPath   = require('./lib/path');

const Electron       = require('electron');
const App            = Electron.app;
const BrowserWindow  = Electron.BrowserWindow;
const IpcMain        = Electron.ipcMain;
const Path           = require('path');
const FS             = require('fs');
const WState         = require('electron-window-state');

const AppDataDirPath = Path.join(App.getPath('appData'), 'mclocks' + (isDebug ? '.dev' : ''));

const config = mConfig.instance(AppDataDirPath);

IpcMain.on("isDebug", (event) => {
  event.returnValue = isDebug;
});

const clocks = config.get("clocks");
IpcMain.on("getClock", (event, arg) => {
  event.returnValue = {
    clocks: clocks,
    formatDateTime: config.get("formatDateTime"),
    localeDateTime: config.get("localeDateTime"),
    fontColor: config.get("fontColor"),
    fontSize: config.get("fontSize"),
    bgColor: config.get("bgColor"),
    onlyText: config.get("onlyText"),
  };
});

let AppWidth = 0;
let AppHeight = 0;
IpcMain.on("fixSize", (event, width, height) => {
  AppWidth = width + (config.get("fontSize") * 2);
  AppHeight = height + 6;
  w.setSize(AppWidth, AppHeight);
  event.returnValue = true;
});

const opacity = config.get("opacity");
const alwaysOnTop = config.get("alwaysOnTop");

let w;

function createWindow() {
  FS.existsSync(AppDataDirPath) || FS.mkdirSync(AppDataDirPath);

  let ws = WState({
    defaultWidth: 1,
    defaultHeight: 1,
    path: AppDataDirPath,
    file: 'window-state.json',
  });

  w = new BrowserWindow({
    x: ws.x,
    y: ws.y,
    width: 1,
    height: 1,
    useContentSize: true,
    frame: false,
    transparent: true,
    opacity: opacity,
    resizable: false,
    hasShadow: false,
    alwaysOnTop: alwaysOnTop,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: mPath.get(__dirname, 'preload.js'),
    },
    icon: mPath.get(__dirname, '../assets/favicon.png'),
  });
  if (isDebug) {
    w.webContents.openDevTools({ mode: 'undocked' });
  }
  w.setMenu(null);
  w.loadURL(`file://${__dirname}/index.html`);
  w.on('closed', () => {
    win = null;
  });
  ws.manage(w);
}

App.on('ready', () => {
  createWindow();
});

App.on('window-all-closed', () => {
  if (process.platform != 'darwin') {
    App.quit();
  }
});

App.on('browser-window-focus', () => {
  adjustAppSize();
});

App.on('activate', () => {
  if (w === null) {
    createWindow();
  }
});

function adjustAppSize() {
  if (AppWidth && AppHeight) {
    w.setSize(AppWidth, AppHeight);
  }
}
