process.once('loaded', () => {
  const {contextBridge, ipcRenderer} = require('electron');
  const {cdate} = require('cdate');

  contextBridge.exposeInMainWorld(
    "mclocks", {
      isDebug: () => {
        return ipcRenderer.sendSync("isDebug");
      },
      getClock: () => {
        return ipcRenderer.sendSync("getClock");
      },
      DT: (timezone, locale, format) => {
        return cdate().tz(timezone).locale(locale).format(format);
      },
      fixSize: (width, height) => {
        return ipcRenderer.sendSync("fixSize", width, height);
      },
    },
  );
});
