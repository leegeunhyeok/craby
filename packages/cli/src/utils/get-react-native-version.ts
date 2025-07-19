import fs from 'node:fs';
import { assert } from 'es-toolkit';
import { getReactNativePath } from './get-react-native-path';
import path from 'node:path';

export function getReactNativeVersion(projectRoot: string): string {
  const reactNativePath = getReactNativePath(projectRoot);
  const packageJsonPath = path.join(reactNativePath, 'package.json');
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

  assert(
    typeof packageJson.version === 'string',
    'Invalid React Native version'
  );

  return packageJson.version;
}
