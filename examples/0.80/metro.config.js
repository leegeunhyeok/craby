const path = require('node:path');
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');

const root = path.resolve(__dirname, '..', '..');

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
      // For resolving peerDependencies
      'react-native': path.join(__dirname, 'node_modules', 'react-native'),
      'react': path.join(__dirname, 'node_modules', 'react'),
      // For resolving workspace packages
      'craby-test': path.join(root, 'examples', 'craby-test', 'src'),
      'craby-modules': path.join(root, 'packages', 'craby-modules', 'dist'),
    },
  },
};

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
