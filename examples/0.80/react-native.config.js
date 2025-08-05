const path = require('node:path');

module.exports = {
  dependencies: {
    'basic-module': {
      root: path.resolve(__dirname, '../basic-module'),
    },
  },
};
