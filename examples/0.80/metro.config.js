const path = require('node:path');
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');

const root = path.resolve(__dirname, '..');

/**
 * Metro configuration
 * https://reactnative.dev/docs/metro
 *
 * @type {import('@react-native/metro-config').MetroConfig}
 */
const config = {
  projectRoot: __dirname,
  watchFolders: [root],
  resolver: {
    extraNodeModules: {
      'craby-test': path.join(root, 'craby-test/src'),
    },
  },
};

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
