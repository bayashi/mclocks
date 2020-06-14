const isDebug = process.argv.includes('--debug');

const Electron       = require('electron');
const App            = Electron.app;
const BrowserWindow  = Electron.BrowserWindow;
const IpcMain        = Electron.ipcMain;
const Path           = require('path');
const FS             = require('fs');
const WState         = require('electron-window-state');
const Store          = require('electron-store');

const AppPath = function (filePath) { return Path.join(__dirname, filePath); }
const AppDataDirPath = Path.join(App.getPath('appData'), 'mclocks' + (isDebug ? '.dev' : ''));

const config = new Store({
  cwd: AppDataDirPath,
  name: 'config',
  defaults: {
    clocks: [
      { name: "Tokyo", timezone: "Asia/Tokyo" },
      { name: "UTC",   timezone: "UTC" },
    ],
    formatDateTime: "MM-DD ddd HH:mm",
    localeDateTime: "en",
    opacity: 1.0,
    fontColor: '#fff',
    fontSize: 14,
    bgColor: '#161',
    alwaysOnTop: false,
  },
  // https://github.com/sindresorhus/electron-store#schema
  schema: {
    clocks: {
      type: "array",
      minItems: 1,
      maxItems: 10,
      items: {
        type: "object",
        minProperties: 2,
        maxProperties: 2,
        properties: {
          name: {
            type: "string",
            minLength: 1,
            maxLength: 10,
            regexp: '/^[a-z0-9\-]+$/',
          },
          // https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
          timezone: {
            type: "string",
            minLength: 1,
            maxLength: 27,
            regexp: '/^[a-z0-9\/\_]+$/',
          },
        },
      }
    },
    // https://momentjs.com/docs/#/parsing/string-format/
    formatDateTime: {
      type: "string",
      minLength: 1,
      maxLength: 50,
    },
    // https://github.com/moment/moment/tree/develop/locale
    localeDateTime: {
      type: "string",
      regexp: '/[a-z]+(-[a-z]+)?/',
      minLength: 2,
      maxLength: 8,
    },
    opacity: {
      type: "number",
      minimum: 0.1,
      maximum: 1.0,
    },
    fontColor: {
      type: "string",
      regexp: '/^#[a-fA-F0-9]+$/',
      minLength: 4,
      maxLength: 7,
    },
    fontSize: {
      type: "number",
      minimum: 8,
      maximum: 36,
    },
    bgColor: {
      type: "string",
      regexp: '/^#[a-fA-F0-9]+$/',
      minLength: 4,
      maxLength: 7,
    },
    alwaysOnTop: {
      type: "boolean",
    },
  },
});

const clocks = config.get("clocks");
IpcMain.on("getClock", (event, arg) => {
  event.returnValue = {
    isDebug: isDebug,
    clocks: clocks,
    formatDateTime: config.get("formatDateTime"),
    localeDateTime: config.get("localeDateTime"),
    fontColor: config.get("fontColor"),
    fontSize: config.get("fontSize"),
    bgColor: config.get("bgColor"),
  };
});
IpcMain.on("fixWidth", (event, width, height) => {
  w.setSize(width + (config.get("fontSize") * 2), height + 6);
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
      preload: AppPath('preload.js'),
    },
    icon: AppPath('../assets/favicon.png'),
  });
  if (isDebug) {
    w.webContents.openDevTools();
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

App.on('activate', () => {
  if (w === null) {
    createWindow();
  }
});
