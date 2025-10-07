import fs from 'node:fs';
import path from 'node:path';
import { createResolver } from './resolver';

export function getConfig(rootDir: string) {
  return {
    projectRoot: rootDir,
    watchFolders: [getWorkspaceRoot(rootDir)],
    resolver: {
      resolveRequest: createResolver({ rootPath: rootDir }),
    },
  };
}

function getWorkspaceRoot(rootDir: string) {
  while (path.dirname(rootDir) !== rootDir) {
    const packageJsonPath = path.join(rootDir, 'package.json');
    if (fs.existsSync(packageJsonPath)) {
      const rawPackageJson = fs.readFileSync(packageJsonPath, 'utf8');
      const packageJson = JSON.parse(rawPackageJson);
      const isWorkspaceRoot = Array.isArray(packageJson.workspaces);

      if (isWorkspaceRoot) {
        return rootDir;
      }
    }
    rootDir = path.dirname(rootDir);
  }
  return null;
}
