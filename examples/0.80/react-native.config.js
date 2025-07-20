const path = require('node:path');

module.exports = {
  dependencies: {
    'craby-calculator': {
      root: path.resolve(__dirname, '../calculator'),
    },
  },
};
