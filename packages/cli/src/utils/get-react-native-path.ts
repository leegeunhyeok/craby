import path from 'node:path';
import { getRequire } from './get-require';

export function getReactNativePath(projectRoot?: string) {
  const require = getRequire();
  const reactNativePath = require.resolve('react-native/package.json', {
    paths: projectRoot ? [projectRoot] : undefined,
  });

  return path.dirname(reactNativePath);
}
