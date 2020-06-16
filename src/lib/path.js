const Path = require('path');

function get(base, paths) {
  return Path.join(base, paths);
}

module.exports = {
  get: get,
};
