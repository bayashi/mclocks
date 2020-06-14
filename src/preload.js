process.once('loaded', () => {
  const {contextBridge, ipcRenderer} = require('electron');
  const Timezone = require('moment-timezone');

  contextBridge.exposeInMainWorld(
    "mclocks", {
      getClock: () => {
        return ipcRenderer.sendSync("getClock");
      },
      Moment: (timezone) => {
        const tz = Timezone.tz(timezone);
        return tz.toArray().concat(tz.day());
      },
      fixWidth: (width, height) => {
        return ipcRenderer.sendSync("fixWidth", width, height);
      },
    },
  );
});
