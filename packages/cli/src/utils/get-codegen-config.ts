import fs from 'node:fs';
import path from 'node:path';
import type { Config } from '../types';

export function getCodegenConfig(projectRoot: string): Config | null {
  const packageJsonPath = path.join(projectRoot, 'package.json');
  const rawPackageJson = fs.readFileSync(packageJsonPath, 'utf8');
  const packageJson = JSON.parse(rawPackageJson);

  return packageJson.codegenConfig ?? null;
}
