# @craby/devkit

Development kit for Craby

## Installation

```bash
npm install @craby/devkit
# or
pnpm add @craby/devkit
# or
yarn add @craby/devkit
```

## Example

### React Native

```js
// react-native.config.js
const path = require('node:path');
const { withWorkspaceModule } = require('@craby/devkit');

const modulePackagePath = path.resolve(__dirname, '../craby-test');
const config = {};

module.exports = withWorkspaceModule(config, modulePackagePath);
```

### Metro

```js
// metro.config.js
const { getMetroConfig } = require('@craby/devkit');
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');

/**
 * Metro configuration
 * https://reactnative.dev/docs/metro
 *
 * @type {import('metro-config').MetroConfig}
 */
const config = getMetroConfig(__dirname);

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
```

Visit [https://craby.rs](https://craby.rs) for full documentation.

## License

[MIT](LICENSE)
