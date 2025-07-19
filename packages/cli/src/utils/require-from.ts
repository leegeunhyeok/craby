import path from 'node:path';
import { getRequire } from './get-require';

export function createRequireFrom(basePath: string) {
  const require = getRequire();

  return <T>(request: string) => require(path.join(basePath, request)) as T;
}
