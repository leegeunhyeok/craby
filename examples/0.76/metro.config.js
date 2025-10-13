const { getConfig } = require('shared');
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');

/**
 * Metro configuration
 * https://reactnative.dev/docs/metro
 *
 * @type {import('metro-config').MetroConfig}
 */
const config = getConfig(__dirname);

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
