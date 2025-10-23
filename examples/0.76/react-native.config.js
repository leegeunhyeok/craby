const path = require('node:path');
const { withWorkspaceModule } = require('@craby/devkit');

const modulePackagePath = path.resolve(__dirname, '../craby-test');
const config = {};

module.exports = withWorkspaceModule(config, modulePackagePath);
