const { getMetroConfig } = require('@craby/devkit');
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');

/**
 * Metro configuration
 * https://reactnative.dev/docs/metro
 *
 * @type {import('@react-native/metro-config').MetroConfig}
 */
const config = getMetroConfig(__dirname);

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
