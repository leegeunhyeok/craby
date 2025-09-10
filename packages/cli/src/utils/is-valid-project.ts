import fs from 'node:fs';
import path from 'node:path';

export function isValidProject(projectRoot: string) {
  try {
    return isValidProjectImpl(projectRoot);
  } catch {
    return false;
  }
}

function isValidProjectImpl(projectRoot: string) {
  return Boolean(fs.existsSync(path.join(projectRoot, 'craby.toml')));
}
