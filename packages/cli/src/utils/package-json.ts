import fs from 'node:fs';
import path from 'node:path';

export function getPackageJsonPath(projectRoot: string) {
  return path.join(projectRoot, 'package.json');
}

export function getPackageJson(projectRoot: string) {
  return JSON.parse(fs.readFileSync(getPackageJsonPath(projectRoot), 'utf8'));
}
