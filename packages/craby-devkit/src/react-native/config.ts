import fs from 'node:fs';
import path from 'node:path';

type ReactNativeConfig = any;

export function withWorkspaceModule(config: ReactNativeConfig, modulePackagePath: string) {
  const rawPackageJson = fs.readFileSync(path.join(modulePackagePath, 'package.json'), 'utf8');
  const packageJson = JSON.parse(rawPackageJson);
  const name = packageJson.name;

  return {
    ...config,
    dependencies: {
      ...config.dependencies,
      [name]: {
        root: modulePackagePath,
      },
    },
  };
}
