import path from 'node:path';
import { getPackageJson } from './package-json';

export function getCommonOptions() {
  const projectRoot = process.cwd();
  const packageJson = getPackageJson(projectRoot);

  return {
    projectRoot,
    templateBasePath: path.resolve(import.meta.dirname, '..', 'templates'),
    packageName: packageJson.name,
  };
}
