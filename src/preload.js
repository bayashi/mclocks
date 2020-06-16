process.once('loaded', () => {
  const {contextBridge, ipcRenderer} = require('electron');
  const Timezone = require('moment-timezone');

  contextBridge.exposeInMainWorld(
    "mclocks", {
      isDebug: () => {
        return ipcRenderer.sendSync("isDebug");
      },
      getClock: () => {
        return ipcRenderer.sendSync("getClock");
      },
      Moment: (timezone, locale, format) => {
        return Timezone.tz(timezone).locale(locale).format(format);
      },
      fixWidth: (width, height) => {
        return ipcRenderer.sendSync("fixWidth", width, height);
      },
    },
  );
});
