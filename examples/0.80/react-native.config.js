const path = require('node:path');

module.exports = {
  dependencies: {
    ...(process.env.E2E === '1' ? null : {
      'craby-test': {
        root: path.resolve(__dirname, '../craby-test'),
      },
    }),
  },
};
