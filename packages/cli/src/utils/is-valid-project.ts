import path from 'node:path';
import fs from 'node:fs';

export function isValidProject(projectRoot: string) {
  try {
    return isValidProjectImpl(projectRoot);
  } catch {
    return false;
  }
}

function isValidProjectImpl(projectRoot: string) {
  const packageJsonPath = path.join(projectRoot, 'package.json');
  const rawPackageJson = fs.readFileSync(packageJsonPath, 'utf-8');
  const packageJson = JSON.parse(rawPackageJson);

  return 'codegenConfig' in packageJson;
}
