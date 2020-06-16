const mConstants = require('./constants');

const Store = require('electron-store');

function instance (AppDataDirPath) {
  return new Store({
    cwd: AppDataDirPath,
    name: 'config',
    defaults: mConstants.defaultConfig,
    schema: {
      clocks: mConstants.clocksSchema,
      ...mConstants.optionsSchema
    },
  });
}

module.exports = {
  instance: instance,
};
